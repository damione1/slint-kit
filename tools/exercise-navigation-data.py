#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise the native showcase's navigation/data flows through embedded Slint MCP.

Build with SLINT_EMIT_DEBUG_INFO=1 and --features slint/mcp, then pass --binary.
Only launch when no other native capture is running: each window must stay active.
This is native event-route evidence, not VoiceOver or IME qualification.
"""
import argparse
import base64
from contextlib import contextmanager, nullcontext
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import subprocess
import sys
import time
import urllib.error

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'docs/capture'))
from mcp import call, value  # noqa: E402

KEY = {'tab': '\t', 'backtab': '\x19', 'escape': '\x1b', 'return': '\n',
       'right': '\uf703', 'down': '\uf701', 'end': '\uf72b'}


class NativeSession:
    def __init__(self, port, process, folder):
        self.port, self.process, self.folder = port, process, folder
        self.window = value(port, 'list_windows')['windowHandles'][0]
        self.root = value(port, 'get_window_properties', windowHandle=self.window)['rootElementHandle']
        self.actions = []
        self.assertions = []
        self.frames = []
        tree = value(port, 'get_element_tree', elementHandle=self.root, maxElements=20)
        if tree.get('debugInfoMissing'):
            raise RuntimeError('Rebuild the showcase with SLINT_EMIT_DEBUG_INFO=1')
        call(port, 'start_event_recording')

    def props(self, handle):
        return value(self.port, 'get_element_properties', elementHandle=handle)

    def handles(self, component=None, scope=None):
        roots = [scope] if scope else [
            value(self.port, 'get_window_properties', windowHandle=window)['rootElementHandle']
            for window in value(self.port, 'list_windows')['windowHandles']]
        found = []
        for root in roots:
            query = [{'matchDescendants': True}]
            if component:
                query.append({'matchElementTypeNameOrBase': component})
            found += value(self.port, 'query_element_descendants', elementHandle=root,
                           queryStack=query, findAll=True).get('elementHandles', [])
        return found

    def find(self, label, component=None, scope=None, enabled=None):
        for handle in self.handles(component, scope):
            props = self.props(handle)
            if (props.get('accessibleLabel') == label
                    and (enabled is None or props.get('accessibleEnabled', False) == enabled)
                    and props.get('computedOpacity', 1) > 0
                    and props.get('size', {}).get('height', 0) > 0):
                return handle
        raise AssertionError(f'Missing {component or "element"}: {label}')

    def component(self, name):
        return next(handle for handle in self.handles(name)
                    if self.props(handle).get('computedOpacity', 1) > 0)

    def texts(self):
        labels = []
        for window in value(self.port, 'list_windows')['windowHandles']:
            root = value(self.port, 'get_window_properties', windowHandle=window)['rootElementHandle']
            tree = value(self.port, 'get_element_tree', elementHandle=root, maxElements=1000)
            labels.extend(element.get('accessibleLabel', '') for element in tree['elements']
                          if element.get('computedOpacity', 1) > 0)
        return labels

    def expect(self, label, *, contains=False):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if any(label in text if contains else label == text for text in self.texts()):
                self.assertions.append(label)
                return
            time.sleep(0.05)
        raise AssertionError(f'Expected visible outcome: {label}')

    def check(self, condition, description):
        if not condition:
            raise AssertionError(description)
        self.assertions.append(description)

    def reveal(self, handle):
        # Use the outer content pane's blank trailing gutter, so a wheel event
        # cannot accidentally scroll a nested table or article list instead.
        found = value(self.port, 'find_elements_by_id', windowHandle=self.window,
                      elementsId='Showcase::page-scroll').get('elementHandles', [])
        if not found:
            return
        viewport = self.props(found[0])
        for _ in range(3):
            props = self.props(handle)
            pos, size = props['absolutePosition'], props['size']
            vpos, vsize = viewport['absolutePosition'], viewport['size']
            if pos['y'] >= vpos['y'] + 8 and pos['y'] + size['height'] <= vpos['y'] + vsize['height'] - 8:
                return
            delta = vpos['y'] + 24 - pos['y']
            call(self.port, 'dispatch_pointer_scroll', windowHandle=self.window,
                 position={'x': vpos['x'] + vsize['width'] - 20,
                           'y': vpos['y'] + vsize['height'] / 2}, deltaY=delta)
            time.sleep(0.1)

    def click(self, handle):
        self.reveal(handle)
        self.actions.append({'action': 'pointer click', 'label': self.props(handle).get('accessibleLabel', '')})
        call(self.port, 'click_element', elementHandle=handle)
        time.sleep(0.08)

    def key(self, name):
        self.actions.append({'action': 'key', 'key': name})
        call(self.port, 'dispatch_key_event', windowHandle=self.window, text=KEY.get(name, name))
        time.sleep(0.08)

    def set_text(self, handle, text):
        self.actions.append({'action': 'native accessible value', 'value': text})
        call(self.port, 'set_element_value', elementHandle=handle, value=text)
        time.sleep(0.08)

    def screenshot(self, name):
        # Capture settled states after theme-driven fades and layout animations.
        time.sleep(0.35)
        block = next(block for block in call(self.port, 'take_screenshot', windowHandle=self.window)
                     if block['type'] == 'image')
        data = base64.b64decode(block['data'], validate=True)
        path = self.folder / f'{name}.png'
        path.write_bytes(data)
        self.frames.append({'file': path.name, 'sha256': hashlib.sha256(data).hexdigest()})

    def finish(self):
        events = value(self.port, 'stop_event_recording')
        if events.get('droppedCount', 0) or events.get('unknownEventCount', 0):
            raise RuntimeError('Native event evidence is incomplete')
        return {'pid': self.process.pid, 'actions': self.actions,
                'assertions': self.assertions, 'frames': self.frames, 'events': events,
                'windowProperties': value(self.port, 'get_window_properties', windowHandle=self.window)}


@contextmanager
def preserve_clipboard():
    # Keep the shared clipboard in memory only, never in logs or evidence.
    original = subprocess.check_output(['pbpaste'])
    try:
        yield
    finally:
        subprocess.run(['pbcopy'], input=original, check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


@contextmanager
def launch(binary, folder, options):
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    environment = {**os.environ, 'SLINT_MCP_PORT': str(port), 'SLINT_EMIT_DEBUG_INFO': '1',
                   'SLINT_BACKEND': 'winit-skia'}
    with (folder / 'native.log').open('w') as log:
        process = subprocess.Popen([str(binary), *options], cwd=ROOT, env=environment,
                                   stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 30
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f'Native process exited; see {folder / "native.log"}')
                try:
                    if value(port, 'list_windows').get('windowHandles'):
                        break
                except (urllib.error.URLError, TimeoutError, ConnectionError):
                    pass
                if time.monotonic() >= deadline:
                    raise TimeoutError('Slint MCP did not start; compile with --features slint/mcp')
                time.sleep(0.1)
            time.sleep(1)
            yield NativeSession(port, process, folder)
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def navigation(session):
    menu = session.component('KitMenu')
    session.click(session.find('Collections', 'KitNavigationAction', menu))
    session.expect('Collections / Overview')
    tabs = session.component('KitTabs')
    session.click(session.find('Activity', 'KitNavigationAction', tabs))
    session.expect('Collections / Activity')
    session.key('right')
    session.key('return')
    session.expect('Collections / People')
    archive = session.find('Archive', 'KitNavigationAction', menu)
    session.check(not session.props(archive).get('accessibleEnabled', False), 'Archive remains disabled')
    call(session.port, 'invoke_accessibility_action', elementHandle=archive, action='Default_')
    session.expect('Collections / People')
    session.screenshot('menu-tabs-keyboard')


def notifications(session):
    center = session.component('KitNotificationCenter')
    mark = session.find('Mark read', 'NotificationAction', center, enabled=True)
    before = session.props(mark)
    session.click(mark)
    session.expect('Read')
    read = session.find('Read', 'NotificationAction', center)
    after = session.props(read)
    session.check(before['absolutePosition'] == after['absolutePosition'] and before['size'] == after['size'],
                  'Mark read retains its action slot')
    session.click(session.find('Open preview', 'NotificationAction', center, enabled=True))
    session.expect('Back to messages')
    session.screenshot('notification-preview')
    session.click(session.find('Back to messages', 'KitButton'))
    center = session.component('KitNotificationCenter')
    session.click(session.find('Dismiss', 'NotificationAction', center, enabled=True))
    time.sleep(0.25)
    session.check('Darlene reviewed three findings' not in session.texts(), 'Dismiss removes the captured notice')
    session.key('escape')
    session.key('return')
    session.component('KitNotificationCenter')
    session.assertions.append('Escape closes and Return reopens from the restored notification trigger')
    session.screenshot('notification-return-focus')


def drawer(session):
    session.key('tab')
    editor = session.find('Collection name', 'TextInput')
    session.set_text(editor, '')
    session.click(session.find('Save settings', 'KitButton'))
    session.expect('Enter a collection name.')
    session.key('Native quality collection')
    session.click(session.find('Save settings', 'KitButton'))
    session.expect('Saved: Native quality collection', contains=True)
    session.key('return')
    session.find('Collection name', 'TextInput')
    session.key('backtab')
    session.key('escape')
    session.expect('Saved: Native quality collection', contains=True)
    session.screenshot('drawer-validation-save-restoration')


def table(session):
    table_handle = session.component('KitDataTable')
    row = session.find('Field notebook 00001', 'DataAction', table_handle)
    call(session.port, 'hover_element', elementHandle=row)
    session.screenshot('table-hover')
    session.click(row)
    selected = [session.props(handle) for handle in session.handles('DataAction', table_handle)]
    session.check(any(props.get('accessibleItemSelected', False)
                      and props.get('accessibleLabel', '').endswith('00001') for props in selected),
                  'Pointer selection retains product #1')
    session.key('end')
    selected = [session.props(handle) for handle in session.handles('DataAction', table_handle)]
    session.check(any(props.get('accessibleItemSelected', False)
                      and props.get('accessibleLabel', '').endswith('10000') for props in selected),
                  'End selects the stable final product #10000')
    table_handle = session.component('KitDataTable')
    session.click(session.find('Product', 'KitPressable', table_handle))
    session.screenshot('table-keyboard-end-sort')
    delegates = [session.props(handle) for handle in session.handles('DataAction', table_handle)]
    (session.folder / 'selected-row-properties.json').write_text(json.dumps(delegates, indent=2) + '\n')
    # MCP can report the same delegate through several inherited debug paths.
    # Compare the unique accessible row projections, not that traversal count.
    selected = list({(props.get('accessibleItemIndex'), props.get('accessibleLabel')): props
                     for props in delegates if props.get('accessibleItemSelected', False)}.values())
    session.check(len(selected) == 1 and selected[0]['accessibleLabel'].endswith('10000'),
                  'Sorting retains the selected product identity')


def article(session):
    session.click(session.find('Close preview', 'KitButton'))
    session.key('down')
    session.expect('Elliot’s field journal · 3')
    session.find('Close preview', 'KitButton')
    session.screenshot('article-keyboard-preview')


def document(session):
    text_view = session.component('KitVirtualTextView')
    rows = session.handles('DataAction', text_view)
    session.check(bool(rows) and len(rows) < 40, 'Document mounts a bounded visible set of native delegates')
    session.click(rows[0])
    editor = session.find('Edit selected line', 'TextInput', text_view)
    session.click(editor)
    session.set_text(editor, '')
    session.key('Native edited line é漢')
    session.key('return')
    session.expect('saved. The other 49,999 lines are unchanged.', contains=True)
    session.click(session.find('Copy', 'KitButton', text_view))
    session.expect('copied.', contains=True)
    session.screenshot('document-edit-save-copy')


def custom_select(session):
    surface = session.component('KitSelectSurface')
    session.screenshot('expressive-select')
    rows = session.handles('KitOption', surface)
    session.check(bool(rows), 'The expressive select mounts option rows')
    inside = {'x': True, 'y': True}
    avatar_count = 0
    for row in rows:
        bounds = session.props(row)
        for avatar in session.handles('Avatar', row):
            avatar_count += 1
            child = session.props(avatar)
            for axis, dimension in [('x', 'width'), ('y', 'height')]:
                start = child['absolutePosition'][axis]
                end = start + child['size'][dimension]
                low = bounds['absolutePosition'][axis]
                high = low + bounds['size'][dimension]
                inside[axis] = inside[axis] and start >= low - 1 and end <= high + 1
    session.check(avatar_count > 0, 'The expressive rows contain their custom avatars')
    for axis in inside:
        session.check(inside[axis], f'Every expressive avatar stays inside its row on {axis}')


def modal_background(session):
    sidebar = session.find('Text fields', 'KitPressable')
    header_labels = {'Product examples', 'Component gallery', 'Light', 'Dark', 'Indigo', 'Teal',
                     'Text 100%', 'Text 200%', 'Compact', 'Standard', 'Comfortable',
                     'Motion on', 'Motion off'}
    background = [session.props(handle) for handle in session.handles('KitButton')
                  if session.props(handle).get('accessibleLabel') in header_labels]
    pane = value(session.port, 'find_elements_by_id', windowHandle=session.window,
                 elementsId='Showcase::page-scroll')['elementHandles'][0]
    def signature(props):
        return json.dumps([props.get('typeNamesAndIds'), props.get('accessibleLabel'),
                           props.get('absolutePosition'), props.get('size')], sort_keys=True)
    drawer = session.component('KitDrawer')
    foreground = {signature(session.props(handle)) for handle in session.handles('KitPressable', drawer)}
    all_background = [session.props(handle) for handle in session.handles('KitPressable')
                      if signature(session.props(handle)) not in foreground]
    before = session.props(pane).get('accessibleLabel')
    sidebar_properties = session.props(sidebar)
    call(session.port, 'invoke_accessibility_action', elementHandle=sidebar, action='Default_')
    time.sleep(0.1)
    after = session.props(pane).get('accessibleLabel')
    diagnostic = {'sidebar': sidebar_properties, 'headerButtons': background,
                  'allBackgroundActions': all_background,
                  'pageBeforeAction': before, 'pageAfterAction': after}
    (session.folder / 'background-action.json').write_text(json.dumps(diagnostic, indent=2) + '\n')
    session.check(not sidebar_properties.get('accessibleEnabled', False),
                  'Sidebar navigation is disabled while the drawer is modal')
    session.check(bool(background) and all(not button.get('accessibleEnabled', False) for button in background),
                  'Preview header actions are disabled while the drawer is modal')
    session.check(bool(all_background) and all(not item.get('accessibleEnabled', False) for item in all_background),
                  'Every mounted background action is ineligible while drawer actions remain active')
    for handle in session.handles('KitPressable'):
        if signature(session.props(handle)) not in foreground:
            call(session.port, 'invoke_accessibility_action', elementHandle=handle, action='Default_')
    session.check(before == after and session.props(pane).get('accessibleLabel') == before,
                  'Background semantic actions cannot replace the modal opener page')
    content = value(session.port, 'find_elements_by_id', windowHandle=session.window,
                    elementsId='Showcase::page-content')['elementHandles'][0]
    panel = session.props(session.handles('KitDialogPanel', drawer)[0])
    viewport = session.props(pane)
    before_y = session.props(content)['absolutePosition']['y']
    call(session.port, 'dispatch_pointer_scroll', windowHandle=session.window,
         position={'x': (viewport['absolutePosition']['x'] + panel['absolutePosition']['x']) / 2,
                   'y': viewport['absolutePosition']['y'] + viewport['size']['height'] / 2}, deltaY=-240)
    time.sleep(0.1)
    session.check(session.props(content)['absolutePosition']['y'] == before_y,
                  'A wheel event on the drawer scrim does not scroll the background')
    session.screenshot('modal-background-ineligible')
    session.key('escape')
    session.check(session.props(sidebar).get('accessibleEnabled', False),
                  'Closing the drawer restores background navigation eligibility')
    call(session.port, 'invoke_accessibility_action', elementHandle=sidebar, action='Default_')
    session.expect('Text fields')
    session.check(session.props(pane).get('accessibleLabel') == 'Text fields',
                  'The same sidebar semantic action works after modal closure')


def wheel_fixture(binary, output, options):
    folder = output / 'modal-wheel-fixture'
    folder.mkdir(parents=True, exist_ok=True)
    source = (ROOT / 'tests/ui/native-dialog.slint').read_text()
    imports = next(line for line in source.splitlines() if line.startswith('import '))
    imports = imports.replace('import {', 'import { KitTheme, KitThemes,')
    source = '// SPDX-License-Identifier: GPL-3.0-or-later\n' + imports + '\n' + source[source.index('export component ModalWheelFixture'):]
    source = source.replace('export component ModalWheelFixture', 'export component Example')
    source = source.replace('width: 640px; height: 480px;',
                            'width: 640px; height: 480px; in-out property <KitTheme> theme: KitThemes.light;')
    source = source.replace('background := KitScrollArea {',
                            'background := KitScrollArea { theme: root.theme; accessible-role: groupbox; accessible-label: "Background offset " + (self.content-y / 1px);')
    source = source.replace('inside := KitScrollArea {',
                            'inside := KitScrollArea { theme: root.theme; accessible-role: groupbox; accessible-label: "Inner offset " + (self.content-y / 1px);')
    source = source.replace('KitDialogSurface {', 'KitDialogSurface { theme: root.theme;')
    source = source.replace('KitDialogPanel {', 'KitDialogPanel { theme: root.theme;')
    path = folder / 'wheel-fixture.slint'
    path.write_text(source)
    with launch(binary, folder, [str(path.with_suffix('')), '--show', *options]) as session:
        def handle(name):
            return value(session.port, 'find_elements_by_id', windowHandle=session.window,
                         elementsId='Example::' + name)['elementHandles'][0]
        background, inside = handle('background'), handle('inside')
        before = session.props(background)['accessibleLabel']
        call(session.port, 'dispatch_pointer_scroll', windowHandle=session.window,
             position={'x': 48, 'y': 48}, deltaY=-120)
        time.sleep(0.1)
        session.check(session.props(background)['accessibleLabel'] == before,
                      'The native modal scrim preserves the background scroll offset')
        call(session.port, 'scroll_element', elementHandle=inside, deltaY=-120)
        time.sleep(0.1)
        session.check(float(session.props(inside)['accessibleLabel'].removeprefix('Inner offset ')) < 0,
                      'The native panel viewport still receives wheel events')
        session.check(session.props(background)['accessibleLabel'] == before,
                      'Scrolling the native panel never scrolls its background')
        session.screenshot('wheel-isolation')
        result = session.finish()
    result['fixtureSha256'] = hashlib.sha256(source.encode()).hexdigest()
    result['binarySha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
    (folder / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print('PASS native modal wheel fixture: 3 outcomes', flush=True)
    return result


FLOWS = {
    'navigation': (['--page', 'navigation'], navigation),
    'notifications': (['--scene', 'notifications-unread'], notifications),
    'drawer': (['--scene', 'drawer-editing'], drawer),
    'table': (['--page', 'data'], table),
    'article': (['--scene', 'article-preview'], article),
    'document': (['--page', 'data', '--position', 'end'], document),
    'custom-select': (['--select', 'expressive'], custom_select),
    'modal-background': (['--scene', 'drawer-editing'], modal_background),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--capture-binary', type=Path, help='Also exercise the overflowing native modal fixture with kit-docs-capture')
    parser.add_argument('--flows', nargs='+', choices=FLOWS,
                        default=['navigation', 'notifications', 'drawer', 'table', 'article', 'document'])
    parser.add_argument('--dark', action='store_true')
    parser.add_argument('--text-200', action='store_true')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = {}
    for name in args.flows:
        folder = output / name
        folder.mkdir(parents=True, exist_ok=True)
        options, exercise = FLOWS[name]
        options = list(options) + (['--dark'] if args.dark else []) + (['--text-200'] if args.text_200 else [])
        clipboard = preserve_clipboard() if name == 'document' else nullcontext()
        with clipboard, launch(binary, folder, options) as session:
            try:
                exercise(session)
                results[name] = session.finish()
            except Exception:
                session.screenshot('failure')
                (folder / 'failure-elements.json').write_text(json.dumps(session.texts(), indent=2) + '\n')
                raise
        (folder / 'result.json').write_text(json.dumps(results[name], indent=2) + '\n')
        print(f'PASS native {name}: {len(results[name]["assertions"])} outcomes', flush=True)
    if args.capture_binary:
        fixture_options = (['--dark'] if args.dark else []) + (['--text-200'] if args.text_200 else [])
        results['modal-wheel-fixture'] = wheel_fixture(args.capture_binary.resolve(strict=True), output, fixture_options)
    report = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'sources': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                          for directory in ['crates/kit-slint/ui', 'showcase/ui', 'showcase/src']
                          for path in sorted((ROOT / directory).rglob('*')) if path.is_file()},
              'backend': 'winit', 'renderer': 'Skia', 'system': platform.platform(),
              'device': subprocess.check_output(['sysctl', '-n', 'hw.model'], text=True).strip(),
              'dark': args.dark, 'textScale': 2 if args.text_200 else 1,
              'qualification': 'Actual native MCP pointer, keyboard and semantic value events. Does not qualify VoiceOver, IME or other devices.',
              'flows': results}
    (output / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
