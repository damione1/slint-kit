#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise catalogue feedback and inspect its actual macOS accessibility tree."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_quality', ROOT / 'tools/exercise-navigation-data.py')
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


def gallery(session):
    session.click(session.find('Run action', 'KitButton'))
    session.expect('Actions completed: 1')
    session.key(' ')
    session.expect('Actions completed: 2')
    session.check(not session.props(session.find('Disabled', 'KitButton')).get('accessibleEnabled', False),
                  'Disabled control is exposed as disabled')
    session.check(not session.props(session.find('Working…', 'KitButton')).get('accessibleEnabled', False),
                  'Busy control is exposed as unavailable')
    output = session.folder / 'macos-accessibility.json'
    subprocess.run(['/usr/bin/swift', str(ROOT / 'tools/audit-native-accessibility.swift'),
                    str(session.process.pid), str(output)], check=True)
    report = json.loads(output.read_text())
    nodes = []

    def walk(node):
        nodes.append(node)
        for child in node.get('children', []):
            walk(child)

    walk(report['tree'])
    labelled = lambda name: [node for node in nodes
                             if name in (node.get('title'), node.get('description'))]
    session.check(any(node.get('role') == 'AXButton' for node in labelled('Run action')),
                  'macOS AX bridge exposes the visible action as AXButton')
    session.check(any(node.get('enabled') is False for node in labelled('Disabled')),
                  'macOS AX bridge preserves the disabled state')
    session.check(any(node.get('enabled') is False for node in labelled('Working…')),
                  'macOS AX bridge preserves the busy action state')
    session.check(any(node.get('selected') is True for node in labelled('Buttons & dialogs')),
                  'macOS AX bridge exposes the selected catalogue page')
    session.check(bool(labelled('Component families')),
                  'macOS AX bridge includes the named navigation region')
    session.screenshot('pointer-space-and-native-accessibility')


def search(session):
    editor = session.find('Search', 'TextInput')
    session.click(editor)
    session.key('Native search query')
    session.check(session.props(editor).get('accessibleValue') == 'Native search query',
                  'Native editor accepts the query')
    session.click(session.find('Clear search', 'KitPressable'))
    session.check(session.props(editor).get('accessibleValue', '') == '',
                  'Clear empties the same native editor')
    session.key('Another query')
    session.check(session.props(editor).get('accessibleValue') == 'Another query',
                  'Clear returns focus to the persistent native editor')
    session.screenshot('search-clear-and-edit')


def toast(session):
    session.find('Could not publish')
    session.click(session.find('Dismiss Could not publish', 'KitIconButton'))
    session.check('Dismiss Could not publish' not in session.texts(),
                  'Visible close action dismisses the notification')
    session.screenshot('toast-dismissed')


def long_label(session):
    label = 'Partager ce nouveau projet avec toutes les personnes de votre équipe et conserver les modifications'
    session.click(session.find(label, 'KitButton'))
    session.key('tab')
    session.key('backtab')
    handles = native.value(session.port, 'find_elements_by_id', windowHandle=session.window,
                           elementsId='KitTooltipLayer::tip').get('elementHandles', [])
    session.check(any(session.props(handle).get('accessibleLabel') == label for handle in handles),
                  'Keyboard focus reveals the complete localized label')
    session.screenshot('long-label-keyboard-tooltip')
    session.key('escape')
    handles = native.value(session.port, 'find_elements_by_id', windowHandle=session.window,
                           elementsId='KitTooltipLayer::tip').get('elementHandles', [])
    session.check(not handles, 'Escape dismisses the keyboard tooltip')
    session.screenshot('long-label-tooltip-dismissed')


FLOWS = {
    'gallery': ([], gallery),
    'search': (['--page', 'profile', '--position', 'end'], search),
    'toast': (['--scene', 'toast-error'], toast),
    'long-label': (['--page', 'gallery', '--text-sample', 'french', '--text-200', '--position', 'end'], long_label),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--flows', nargs='+', choices=FLOWS, default=list(FLOWS))
    parser.add_argument('--dark', action='store_true')
    parser.add_argument('--text-200', action='store_true')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    results = {}
    for name in args.flows:
        folder = args.output.resolve() / name
        folder.mkdir(parents=True, exist_ok=True)
        options, exercise = FLOWS[name]
        options = list(options) + (['--dark'] if args.dark else []) + (['--text-200'] if args.text_200 else [])
        with native.launch(binary, folder, options) as session:
            try:
                exercise(session)
                results[name] = session.finish()
                results[name]['arguments'] = options
            except Exception:
                session.screenshot('failure')
                raise
        (folder / 'result.json').write_text(json.dumps(results[name], indent=2) + '\n')
        print(f'PASS native {name}: {len(results[name]["assertions"])} outcomes', flush=True)
    report = {
        'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
        'sources': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                    for directory in ['crates/kit-slint/ui', 'showcase/ui', 'showcase/src']
                    for path in sorted((ROOT / directory).rglob('*')) if path.is_file()},
        'backend': 'winit', 'renderer': 'Skia',
        'system': platform.platform(),
        'device': subprocess.check_output(['sysctl', '-n', 'hw.model'], text=True).strip(),
        'dark': args.dark, 'textScale': 2 if args.text_200 else 1,
        'qualification': 'Native event routes and actual macOS AX bridge. Does not qualify VoiceOver or IME.',
        'flows': results,
    }
    (args.output / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
