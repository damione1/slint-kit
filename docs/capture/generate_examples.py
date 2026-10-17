#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regenerate complete, intentionally small public-API documentation examples."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
UI = ROOT / 'crates/kit-slint/ui'
public = set()
for group in re.findall(r'export \{([^}]+)\}', (UI / 'kit.slint').read_text()):
    public.update(x.strip() for x in group.split(','))
components = {}
for path in sorted(UI.glob('*.slint')):
    for name, base in re.findall(r'export component (\w+) inherits (\w+)', path.read_text()):
        if name in public:
            components[name] = base

def slug(name):
    return re.sub(r'(?<!^)(?=[A-Z])', '-', name.removeprefix('Kit')).lower()

def txt(s):
    return 'Text { text: "' + s + '"; color: root.theme.text-primary; font-size: 14px * root.theme.text-scale; wrap: word-wrap; }'

def button(s='Continue'):
    return 'KitButton { theme: root.theme; text: "' + s + '"; activated => { root.status = "Action completed"; } }'

bodies = {}
extra = {}
heights = {}
def add(name, body, height=240, properties=''):
    bodies[name] = body
    heights[name] = height
    extra[name] = properties

for name in ['KitButton', 'KitFormButton', 'KitSubmitButton', 'KitResetButton']:
    label = {'KitResetButton':'Reset form', 'KitSubmitButton':'Create account'}.get(name, 'Save changes')
    add(name, f'{name} {{ theme: root.theme; text: "{label}"; activated => {{ root.status = "Action completed"; }} }}')
add('KitPressable', 'KitPressable { theme: root.theme; accessible-name: "Add to collection"; activated => { root.status = "Added to collection"; } Text { text: "Add to collection"; color: root.theme.text-primary; horizontal-alignment: center; vertical-alignment: center; } }')
add('KitIconButton', 'KitIconButton { theme: root.theme; icon: IconKind.plus; accessible-name: "New project"; activated => { root.status = "New project requested"; } }')
add('KitButtonGroup', 'KitButtonGroup { ' + button('Back') + button('Next') + ' }')
add('KitIcon', 'HorizontalLayout { alignment: start; spacing: 24px; KitIcon { kind: IconKind.check; ink: root.theme.accent; } KitIcon { kind: IconKind.search; ink: root.theme.text-primary; } KitIcon { kind: IconKind.warning; ink: root.theme.warning; } }')
add('KitFocusTarget', 'KitFocusTarget { token: 42; generation: 1; request: { present: true, token: 42, generation: 1, request: 1 }; focus-requested => { editor.take-focus(); } editor := KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; } }')

values = {
'KitTextField': ('Project name','Field notes'), 'KitPasswordField':('Password','keep-it-secret'),
'KitSearchField':('Search projects','design'), 'KitEmailField':('Email address','alex@example.com'),
'KitUrlField':('Website','https://example.com'), 'KitTelephoneField':('Phone','+1 514 555 0132'),
'KitTextArea':('Notes','A native editor preserves selection, undo and input methods.'),
'KitOtpField':('Verification code','482913'), 'KitMaskedField':('Reference','AB-2048'),
'KitIntegerField':('Seats','12'), 'KitDecimalField':('Distance','12.75'),
'KitMoneyField':('Budget','129.95'), 'KitPercentField':('Tax rate','8.25'),
'KitQuantityField':('Weight','2.50'), 'KitDurationField':('Duration','01:30'),
'KitNumberStepper':('Guests','3'), 'KitDateField':('Event date','2026-10-17'),
'KitMonthField':('Billing month','2026-10'), 'KitYearField':('Fiscal year','2026'),
'KitIsoWeekField':('Sprint week','2026-W42'), 'KitTimeField':('Start time','09:30')}
for name,(label,value) in values.items():
    suffix = ' prefix: "$";' if name == 'KitMoneyField' else ' suffix: "kg";' if name == 'KitQuantityField' else ''
    suffix += ' step-requested(direction) => { root.status = "Step requested; validate in Rust"; };' if name == 'KitNumberStepper' else ''
    add(name, f'{name} {{ theme: root.theme; label: "{label}"; text: "{value}";{suffix} edited(value) => {{ root.status = "Draft: " + value; }} }}', 300 if name == 'KitTextArea' else 240)

add('KitField', 'KitField { KitFieldLabel { theme: root.theme; text: "Display name"; } KitTextField { theme: root.theme; text: "Alex Morgan"; } KitFieldDescription { theme: root.theme; text: "Visible to your team."; } }')
add('KitFieldLabel', 'KitFieldLabel { theme: root.theme; text: "Email address"; } KitTextField { theme: root.theme; placeholder: "you@example.com"; }')
add('KitFieldDescription', 'KitTextField { theme: root.theme; label: "Username"; text: "alex"; } KitFieldDescription { theme: root.theme; text: "Use 3–24 letters, numbers or underscores."; }')
add('KitFieldError', 'KitTextField { theme: root.theme; label: "Email"; text: "alex@"; validation: ValidationPresentation.invalid; } KitFieldError { theme: root.theme; text: "Enter a complete email address."; }')
add('KitFieldGroup', 'KitFieldGroup { KitTextField { theme: root.theme; label: "First name"; text: "Alex"; } KitTextField { theme: root.theme; label: "Last name"; text: "Morgan"; } }', 330)
add('KitFieldSet', 'KitFieldSet { theme: root.theme; legend: "Shipping address"; KitTextField { theme: root.theme; label: "City"; text: "Montréal"; } KitTextField { theme: root.theme; label: "Postal code"; text: "H2Y 1C6"; } }', 360)
add('KitInputGroup', 'KitInputGroup { theme: root.theme; HorizontalLayout { spacing: 8px; KitInputGroupText { theme: root.theme; text: "https://"; } TextInput { text: "example.com"; color: root.theme.text-primary; } KitInputGroupText { theme: root.theme; text: "/"; } } }')
for name in ['KitInputGroupStart','KitInputGroupEnd','KitInputGroupText']:
    content = f'{name} {{ ' + ('theme: root.theme; text: "USD";' if name=='KitInputGroupText' else 'KitInputGroupText { theme: root.theme; text: "USD"; }') + ' }'
    add(name, 'KitInputGroup { theme: root.theme; HorizontalLayout { spacing: 12px; ' + (content if name!='KitInputGroupEnd' else '') + ' TextInput { text: "129.95"; color: root.theme.text-primary; } ' + (content if name=='KitInputGroupEnd' else '') + ' } }')
add('KitFieldSeparator', 'KitTextField { theme: root.theme; label: "Work email"; text: "alex@example.com"; } KitFieldSeparator { theme: root.theme; } KitTextField { theme: root.theme; label: "Personal email"; placeholder: "Optional"; }', 330)
add('KitFormErrorSummary', 'KitFormErrorSummary { theme: root.theme; errors: [{ target: { present: true, token: 42, generation: 1, request: 1 }, message: "Enter a complete email address." }]; error-activated(request) => { root.status = "Focus field " + request.token; } }')

rows='[{ token: 11, generation: 1, label: "Overview", description: "Project summary", disabled: false }, { token: 23, generation: 1, label: "Activity", description: "Recent changes", disabled: false }, { token: 38, generation: 1, label: "Settings", description: "Project preferences", disabled: false }]'
state='{ open: false, generation: 1, active-index: 0, active: { present: true, token: 11 }, selected: { present: true, token: 11 }, selection-availability: SelectionAvailability.available }'
choice='[{ token: 11, label: "Email", selected: true, disabled: false }, { token: 23, label: "Push", selected: false, disabled: false }, { token: 38, label: "SMS", selected: false, disabled: true }]'
chip='{ token: 11, label: "Design", removable: true }'
for name in ['KitSelect','KitSelectTrigger','KitMenu','KitMenuBar','KitTabs','KitCarousel']:
    attrs = 'rows: '+rows+';' if name != 'KitSelectTrigger' else 'Text { text: "Overview"; color: root.theme.text-primary; vertical-alignment: center; }'
    add(name, f'{name} {{ theme: root.theme; {attrs} state: {state}; activate(token, generation) => {{ root.status = "Activated stable token " + token; }} }}', 360 if name=='KitMenu' else 280)
add('KitSelectKeyboard', 'KitSelectKeyboard { state: '+state.replace('open: false','open: true')+'; navigate(step) => { root.status = "Keyboard navigation requested"; } '+txt('Use arrow keys, Home, End and Enter to navigate a keyed selection.')+ ' }')
add('KitSelectSurface', 'KitSelectSurface { theme: root.theme; height: 220px; state: '+state.replace('open: false','open: true')+'; VerticalLayout { for row in '+rows+': KitOption { theme: root.theme; option-data: row; } } }', 370)
for name in ['KitOption','KitNavigationAction']:
    add(name, name+' { theme: root.theme; option-data: { token: 11, generation: 1, label: "Project overview", description: "Open project details", disabled: false }; selected: true; activate(token, generation) => { root.status = "Activated stable token " + token; } }')
add('KitCheckbox', 'KitCheckbox { theme: root.theme; label: "Send me product updates"; state: CheckState.checked; toggled(state) => { root.status = "Checkbox change requested"; } }')
add('KitSwitch', 'KitSwitch { theme: root.theme; label: "Desktop notifications"; checked: true; toggled(value) => { root.status = value ? "Enable requested" : "Disable requested"; } }')
for name in ['KitRadioGroup','KitSegmentedControl','KitToggleGroup']:
    cb='toggled' if name=='KitToggleGroup' else 'activate'
    add(name, name+' { theme: root.theme; rows: '+choice+'; generation: 1; active-token: 11; '+cb+'(token, generation) => { root.status = "Choice token " + token; } }', 320 if name=='KitRadioGroup' else 240)
add('KitChip', 'KitChip { theme: root.theme; chip: '+chip+'; remove-requested(token) => { root.status = "Remove token " + token; } }')
add('KitMultiSelect', 'KitMultiSelect { theme: root.theme; rows: '+rows+'; chips: ['+chip+']; state: '+state+'; }', 300)
add('KitTagInput', 'KitTagInput { theme: root.theme; label: "Project tags"; chips: ['+chip+', { token: 23, label: "Research", removable: true }]; text: ""; create-requested(value) => { root.status = "Create tag: " + value; } }', 280)
add('KitCombobox', 'KitCombobox { theme: root.theme; label: "Search projects"; rows: '+rows+'; state: '+state+'; selected-label: "Overview"; query: ""; query-edited(value) => { root.status = "Search: " + value; } }')
add('KitSlider', 'KitSlider { theme: root.theme; label: "Volume"; position: 0.65; value-label: "65%"; position-requested(value) => { root.status = "Position requested"; } }')
add('KitRangeSlider', 'KitRangeSlider { theme: root.theme; label: "Price range"; lower-position: 0.2; upper-position: 0.75; lower-label: "$20"; upper-label: "$75"; }')
add('KitRating', 'KitRating { theme: root.theme; label: "Your rating"; generation: 1; active-token: 4; scores: ['+', '.join('{ token: '+str(i)+', label: "'+str(i)+' stars", selected: '+str(i==4).lower()+', disabled: false }' for i in range(1,6))+']; }')

from datetime import date, timedelta
calendar_dates = [date(2026, 9, 28) + timedelta(days=i) for i in range(42)]
days='['+', '.join('{ year: '+str(day.year)+', month: '+str(day.month)+', day: '+str(day.day)+', label: "'+str(day.day)+'", accessible-label: "'+day.strftime('%B %-d, %Y')+'", enabled: '+str(day.month==10).lower()+', selected: '+str(day==date(2026,10,17)).lower()+', today: '+str(day==date(2026,10,17)).lower()+', in-range: false }' for day in calendar_dates)+']'

for name in ['KitCalendar','KitDateRangePicker']:
    attrs='summary: "October 17–21, 2026";' if name=='KitDateRangePicker' else ''
    add(name, name+' { theme: root.theme; title: "October 2026"; days: '+days+'; '+attrs+' choose(day) => { root.status = "Requested day " + day.day; } }', 620)
add('KitDatePicker', 'KitDatePicker { theme: root.theme; label: "Event date"; text: "2026-10-17"; accepted-date: { year: 2026, month: 10, day: 17 }; title: "October 2026"; days: '+days+'; weekdays: ["M", "T", "W", "T", "F", "S", "S"]; }')
add('KitTimePicker', 'KitTimePicker { theme: root.theme; label: "Start time"; text: "09:30"; accepted-time: { hour: 9, minute: 30, second: 0 }; }', 370)
add('KitDateTimeField', 'KitDateTimeField { theme: root.theme; label: "Meeting"; date-text: "2026-10-17"; time-text: "09:30"; }', 330)
add('KitZonedDateTimeField', 'KitZonedDateTimeField { theme: root.theme; label: "Meeting"; date-text: "2026-10-17"; time-text: "09:30"; zone-text: "America/Toronto"; resolution: ZonedTimePresentation.unique; resolution-message: "UTC−04:00"; }', 390)
file_entries='[{ token: 11, generation: 1, name: "project-brief.txt", detail: "Text document · 2.4 KB", removable: true }, { token: 23, generation: 1, name: "meeting-notes.md", detail: "Markdown · 1.2 KB", removable: true }]'
for name in ['KitFileList','KitFilePicker']:
    add(name, name+' { theme: root.theme; entries: '+file_entries+'; remove-requested(token, generation) => { root.status = "Remove file token " + token; } }', 360)
add('KitDirectoryPicker', 'KitDirectoryPicker { theme: root.theme; label: "Export folder"; selected-name: "Documents/Project exports"; browse-requested => { root.status = "Host folder request"; } }', 300)
add('KitFileDropZone', 'KitFileDropZone { theme: root.theme; capability: DropCapability.unsupported; } '+button('Choose files…'), 320)
for name in ['KitColorField','KitColorPicker']:
    attr='text: "#6366F1";' if name=='KitColorField' else 'hex-text: "#6366F1"; red-text: "99"; green-text: "102"; blue-text: "241";'
    add(name, name+' { theme: root.theme; label: "Accent color"; swatch: #6366f1; accepted-color: { red: 99, green: 102, blue: 241, alpha: 255 }; '+attr+' }', 590 if name=='KitColorPicker' else 250)

for name in ['KitContainer','KitStack','KitInline','KitWrap','KitGrid','KitSection','KitSurface','KitScrollArea','KitAppShell']:
    attrs='theme: root.theme;' if name in ['KitSection','KitSurface','KitAppShell'] else ''
    attrs+=' title: "Team workspace"; description: "Compose ordinary children with shared spacing.";' if name=='KitSection' else ''
    if name=='KitGrid':
        children='Row { '+button('Overview')+button('Activity')+' } Row { '+button('Members')+button('Settings')+' }'
    elif name in ['KitInline','KitWrap']:
        children=button('Overview')+button('Activity')+button('Settings')
    else:
        children=txt('Team workspace')+'KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; }'+button('Save project')
    if name=='KitScrollArea':attrs+=' height: 190px;'
    add(name,name+' { '+attrs+children+' }',400)

add('KitBadge', 'HorizontalLayout { alignment: start; spacing: 12px; KitBadge { theme: root.theme; text: "Draft"; } KitBadge { theme: root.theme; text: "Published"; tone: Tone.success; } KitBadge { theme: root.theme; text: "3 updates"; tone: Tone.accent; } }')
add('KitAvatar', 'HorizontalLayout { alignment: start; spacing: 16px; KitAvatar { theme: root.theme; initials: "AM"; accessible-name: "Alex Morgan"; size: 56px; } KitAvatar { theme: root.theme; initials: "JS"; accessible-name: "Jamie Smith"; shape: AvatarShape.rounded; size: 56px; } }')
add('KitCard', 'KitCard { theme: root.theme; KitCardHeader { theme: root.theme; title: "Field notes"; description: "Your team’s shared research workspace."; } KitCardBody { '+txt('Keep observations and decisions together.')+' } KitCardFooter { '+button('Open project')+' } }',340)
add('KitCardHeader', 'KitCardHeader { theme: root.theme; title: "Project settings"; description: "Manage your workspace and collaborators."; }')
add('KitCardBody', 'KitCard { theme: root.theme; KitCardBody { '+txt('A card body groups the primary content.')+' KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; } } }',300)
add('KitCardFooter','KitCard { theme: root.theme; '+txt('Ready to publish your changes?')+'KitCardFooter { '+button('Cancel')+button('Publish')+' } }')
add('KitHero','KitHero { theme: root.theme; title: "Make room for ideas."; description: "A shared workspace for your next project."; '+button('Create workspace')+' }',350)
add('KitFooter','KitFooter { theme: root.theme; '+txt('Slint Kit · Native applications, thoughtfully composed.')+'KitInline { '+button('Documentation')+button('Contact')+' } }',280)
add('KitDivider','KitTextField { theme: root.theme; label: "Email address"; placeholder: "you@example.com"; } KitDivider { theme: root.theme; label: "or continue with"; } '+button('Single sign-on'),300)
add('KitStat','KitStat { theme: root.theme; label: "Active projects"; value: "24"; description: "4 added this month"; }')
add('KitChatBubble','KitChatBubble { theme: root.theme; author: "Alex Morgan"; text: "The new workspace is ready for your feedback."; metadata: "Today, 09:30"; }',280)
add('KitSkeleton','KitSkeleton { theme: root.theme; running: false; height: 22px; } KitSkeleton { theme: root.theme; running: false; height: 16px; width: 340px; } KitSkeleton { theme: root.theme; running: false; height: 16px; width: 260px; }')
add('KitEmptyState','KitEmptyState { theme: root.theme; title: "No projects yet"; description: "Create a project to start collecting your ideas."; '+button('Create project')+' }',340)
add('KitTimeline','KitTimeline { theme: root.theme; entries: [{ title: "Project created", description: "Alex opened the workspace.", metadata: "09:00" }, { title: "Research shared", description: "Jamie added the interview notes.", metadata: "09:30" }, { title: "Ready for review", description: "The team can leave feedback.", metadata: "10:15" }]; }',390)
add('KitSteps','KitSteps { theme: root.theme; entries: [{ token: 11, label: "Account", state: StepState.complete }, { token: 23, label: "Workspace", state: StepState.current }, { token: 38, label: "Invite team", state: StepState.pending }]; }',330)
add('KitAlert','KitAlert { theme: root.theme; severity: AlertSeverity.success; title: "Changes saved"; message: "Your team can now see the updated project."; dismissible: true; dismissed => { root.status = "Dismiss requested"; } }',280)
add('KitSpinner','HorizontalLayout { alignment: start; spacing: 16px; KitSpinner { theme: root.theme; running: false; label: "Loading projects"; } '+txt('Loading projects…')+' }')
add('KitProgress','KitProgress { theme: root.theme; label: "Uploading documents · 65%"; value: 0.65; }')
add('KitRadialProgress','KitRadialProgress { theme: root.theme; label: "Upload progress"; value: 0.72; }')
for name in ['KitTooltip','KitTooltipLayer']:
    add(name,'KitTooltip { theme: root.theme; text: "Create a new project in this workspace."; target-focused: true; viewport-width: root.width; viewport-height: root.height; '+button('New project')+' }',320)
entry='{ token: 11, title: "Project archived", message: "Your notes are safely stored.", action-label: "Undo", severity: AlertSeverity.success, dismissible: true }'
add('KitToast','KitToast { theme: root.theme; entry: '+entry+'; action(token) => { root.status = "Undo requested"; } }',300)
add('KitToastHost','KitToastHost { theme: root.theme; entries: ['+entry+', { token: 23, title: "Invite sent", message: "Jamie can now join the workspace.", action-label: "", severity: AlertSeverity.information, dismissible: true }]; }',440)
add('KitCollapse','KitCollapse { theme: root.theme; title: "What happens after I publish?"; expanded: true; '+txt('Your team receives a notification and can review the latest version.')+' }',300)
add('KitAccordion','KitAccordion { theme: root.theme; rows: [{ item: { token: 11, generation: 1, label: "Project permissions", description: "Members can read and edit all project notes.", disabled: false }, expanded: true }, { item: { token: 23, generation: 1, label: "Notification settings", description: "Choose how you receive updates.", disabled: false }, expanded: false }]; state: '+state+'; }',350)
add('KitBreadcrumbs','KitBreadcrumbs { theme: root.theme; rows: [{ token: 11, generation: 1, label: "Workspace", disabled: false }, { token: 23, generation: 1, label: "Projects", disabled: false }, { token: 38, generation: 1, label: "Field notes", disabled: false }]; }')
add('KitPagination','KitPagination { theme: root.theme; page-index: 2; page-count: 12; }')
add('KitNotificationButton','KitNotificationButton { theme: root.theme; unread-count: 3; activated => { root.status = "Open notification center"; } }')
add('KitNotificationCenter','KitNotificationCenter { theme: root.theme; height: 300px; rows: [{ item: { token: 11, generation: 1, label: "Jamie shared a document", description: "Research notes are ready for review.", disabled: false }, read: false }, { item: { token: 23, generation: 1, label: "Your export is ready", description: "Download the latest project archive.", disabled: false }, read: true }]; }',440)
add('KitDrawer','KitDrawer { theme: root.theme; width: parent.width; height: parent.height; title: "Project details"; state: { open: true, topmost: true, busy: false }; '+txt('Field notes')+'KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; }'+button('Save changes')+' }',460)

add('KitDataTable','KitDataTable { theme: root.theme; height: 250px; generation: 1; columns: [{ token: 11, label: "Project", sortable: true, sort: DataSortDirection.ascending, alignment: DataAlignment.leading }, { token: 23, label: "Status", sortable: true, sort: DataSortDirection.unsorted, alignment: DataAlignment.leading }, { token: 38, label: "Owner", sortable: false, sort: DataSortDirection.unsorted, alignment: DataAlignment.leading }]; rows: [{ token: 101, cells: ["Field notes", "In progress", "Alex"], selected: true, disabled: false }, { token: 205, cells: ["Design system", "Review", "Jamie"], selected: false, disabled: false }, { token: 309, cells: ["Launch plan", "Draft", "Taylor"], selected: false, disabled: false }]; activate(token, generation) => { root.status = "Project token " + token; } }',400)
add('KitContentList','KitContentList { theme: root.theme; height: 260px; generation: 1; items: [{ token: 101, title: "A guide to native design", excerpt: "Build interfaces that feel at home on the desktop.", metadata: "Design · 5 min read", link-label: "Read article", selected: false, disabled: false }, { token: 205, title: "Keep domain values exact", excerpt: "Use validated values and stable application identity.", metadata: "Engineering · 8 min read", link-label: "Read article", selected: false, disabled: false }]; }',410)
add('KitVirtualTextView','KitVirtualTextView { theme: root.theme; height: 230px; generation: 1; lines: [{ token: 101, number: 1, text: "# Field notes", selected: false }, { token: 205, number: 2, text: "A shared space for observations.", selected: true }, { token: 309, number: 3, text: "Keep identities stable as rows move.", selected: false }, { token: 413, number: 4, text: "Use native editing for selected lines.", selected: false }]; }',380)

add('KitDialogSurface','KitDialogSurface { theme: root.theme; width: parent.width; height: parent.height; state: { open: true, topmost: true, busy: false }; KitDialogPanel { theme: root.theme; x: (parent.width - self.width) / 2; y: 60px; width: 420px; height: 220px; accessible-name: "Archive project"; VerticalLayout { padding: 24px; spacing: 16px; '+txt('Archive this project?')+txt('You can restore it later from the archive.')+button('Archive project')+' } } }',400)
for name in ['KitDialogPanel','KitDialogHeader','KitDialogBody','KitDialogFooter']:
    child = button('Cancel')+button('Save changes') if name=='KitDialogFooter' else txt('Project settings') if name=='KitDialogHeader' else 'KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; }' if name=='KitDialogBody' else txt('Archive this project?')+button('Archive project')
    add(name, name+' { theme: root.theme; VerticalLayout { padding: 20px; spacing: 16px; '+child+' } }',320)

# Demonstrate the layout behavior inside finite caller-chosen viewports.
bodies['KitWrap'] = 'KitWrap { '+button('Overview')+button('Activity')+button('Settings')+button('Members')+button('Project settings')+' }'
bodies['KitScrollArea'] = 'KitScrollArea { height: 190px; inset: LayoutSpace.compact; gap: LayoutSpace.regular; KitTextField { theme: root.theme; label: "Project name"; text: "Field notes"; } KitTextField { theme: root.theme; label: "Email address"; text: "alex@example.com"; } KitTextField { theme: root.theme; label: "Website"; text: "https://example.com"; } KitTextField { theme: root.theme; label: "Phone"; text: "+1 514 555 0132"; } KitTextArea { theme: root.theme; label: "Notes"; text: "Keep observations and decisions together."; } '+button('Save project')+' }'

# Dialog slots use intrinsic reading-order content and shared type hierarchy.
dialog_title = 'Text { text: "Archive this project?"; color: root.theme.text-primary; font-size: 20px * root.theme.text-scale; font-weight: 600; wrap: word-wrap; vertical-stretch: 0; }'
dialog_body = txt('You can restore it later from the archive.').replace('wrap: word-wrap;', 'wrap: word-wrap; vertical-stretch: 0;')
dialog_status = 'if root.status != "": Text { text: root.status; color: root.theme.text-secondary; font-size: 12px * root.theme.text-scale; wrap: word-wrap; vertical-stretch: 0; }'
dialog_panel = 'KitDialogPanel { theme: root.theme; width: 420px; height: dialog-content.preferred-height; horizontal-stretch: 0; vertical-stretch: 0; accessible-name: "Archive project"; dialog-content := VerticalLayout { padding: 24px; spacing: 16px; alignment: start; '+dialog_title+dialog_body+button('Archive project')+dialog_status+' } }'
bodies['KitDialogPanel'] = dialog_panel
bodies['KitDialogSurface'] = 'KitDialogSurface { theme: root.theme; width: parent.width; height: parent.height; state: { open: true, topmost: true, busy: false }; '+dialog_panel.replace('width: 420px;', 'width: 420px; x: (parent.width - self.width) / 2; y: (parent.height - self.height) / 2;', 1)+' }'
bodies['KitDialogHeader'] = 'KitDialogHeader { theme: root.theme; vertical-stretch: 0; '+dialog_title+' }'
bodies['KitDialogBody'] = 'KitDialogBody { theme: root.theme; vertical-stretch: 0; '+dialog_body+' }'
bodies['KitDialogFooter'] = 'KitDialogFooter { theme: root.theme; vertical-stretch: 0; '+button('Cancel')+button('Save changes')+' }'

# Primitive rows own identity/interaction; their visual content belongs to callers.
bodies['KitOption'] = bodies['KitOption'][:-2] + ' Text { text: "Project overview"; color: root.theme.text-primary; vertical-alignment: center; font-size: 14px * root.theme.text-scale; } }'
bodies['KitSelectSurface'] = bodies['KitSelectSurface'].replace('option-data: row; }', 'option-data: row; height: 48px; Text { text: row.label; color: root.theme.text-primary; vertical-alignment: center; font-size: 14px * root.theme.text-scale; } }')
bodies['KitSelect'] = bodies['KitSelect'].replace('theme: root.theme;', 'theme: root.theme; placeholder: "Overview";', 1)
bodies['KitSelectKeyboard'] = bodies['KitSelectKeyboard'].replace('KitSelectKeyboard {', 'KitSelectKeyboard { init => { self.focus(); }', 1)
for overlay in ['KitDrawer']:
    body = bodies[overlay]
    marker = ' } } }' if overlay == 'KitDialogSurface' else ' }'
    status = ' if root.status != "": Text { text: root.status; color: root.theme.text-secondary; font-size: 12px * root.theme.text-scale; wrap: word-wrap; }'
    bodies[overlay] = body[:-len(marker)] + status + marker
bodies['KitSlider'] = txt('Volume · 65%') + ' ' + bodies['KitSlider']
bodies['KitRangeSlider'] = txt('Price range · $20–$75') + ' ' + bodies['KitRangeSlider']

# Show the keyboard primitive in its real trigger + keyed option composition.
bodies['KitSelectKeyboard'] = 'KitSelectKeyboard { init => { self.focus(); } state: '+state.replace('open: false', 'open: true')+'; navigate(step) => { root.status = "Keyboard navigation requested"; } activate(token, generation) => { root.status = "Activation requested for token " + token; } VerticalLayout { spacing: 8px; KitSelectTrigger { theme: root.theme; state: '+state.replace('open: false', 'open: true')+'; accessible-name: "Project navigation"; Text { text: "Overview"; color: root.theme.text-primary; vertical-alignment: center; font-size: 14px * root.theme.text-scale; } } Rectangle { background: root.theme.surface-raised; border-color: root.theme.border-subtle; border-width: 1px; border-radius: 8px; VerticalLayout { padding: 6px; spacing: 4px; for row in '+rows+': KitOption { theme: root.theme; option-data: row; active: row.token == 11; selected: row.token == 11; height: 42px; Text { text: row.label; color: root.theme.text-primary; font-size: 14px * root.theme.text-scale; vertical-alignment: center; horizontal-alignment: left; } } } } } }'
for group in ['KitInputGroup', 'KitInputGroupStart', 'KitInputGroupEnd', 'KitInputGroupText']:
    bodies[group] = bodies[group].replace('TextInput { text:', 'TextInput { font-size: 14px * root.theme.text-scale; vertical-alignment: center; text:')
# Observation text describes requests; it does not claim external persistence.
for name in bodies:
    bodies[name] = bodies[name].replace('Action completed', 'Action requested')

# A screenshot window is a caller-chosen viewport, sized to its actual content.
geometry = {name: (420, 180) for name in bodies}
for name in ['KitButton', 'KitFormButton', 'KitSubmitButton', 'KitResetButton', 'KitPressable', 'KitButtonGroup', 'KitCheckbox', 'KitSwitch', 'KitBadge', 'KitAvatar', 'KitSpinner', 'KitIcon']:
    geometry[name] = (360, 160)
geometry.update({
    'KitIconButton': (320, 150), 'KitField': (420, 220),
    'KitTextArea': (420, 240), 'KitFieldDescription': (420, 220),
    'KitFieldError': (420, 220), 'KitFieldGroup': (420, 280),
    'KitFieldSet': (440, 310), 'KitFieldSeparator': (420, 290),
    'KitFormErrorSummary': (420, 180), 'KitRadioGroup': (420, 250),
    'KitRating': (560, 170), 'KitTagInput': (420, 220),
    'KitMultiSelect': (420, 220), 'KitSelectSurface': (420, 320),
    'KitSelectKeyboard': (420, 340), 'KitMenu': (420, 260),
    'KitCalendar': (360, 440), 'KitDateRangePicker': (420, 520),
    'KitTimePicker': (620, 300), 'KitDateTimeField': (440, 290),
    'KitZonedDateTimeField': (480, 390), 'KitFileList': (520, 260),
    'KitFilePicker': (520, 330), 'KitDirectoryPicker': (520, 260),
    'KitFileDropZone': (520, 270), 'KitColorField': (420, 210),
    'KitColorPicker': (480, 550), 'KitContainer': (480, 290),
    'KitStack': (480, 290), 'KitInline': (420, 160),
    'KitWrap': (420, 220), 'KitGrid': (420, 230),
    'KitSection': (480, 350), 'KitSurface': (480, 300),
    'KitScrollArea': (480, 290), 'KitAppShell': (520, 310),
    'KitCard': (480, 300), 'KitCardBody': (440, 260),
    'KitCardFooter': (480, 220), 'KitHero': (520, 300),
    'KitFooter': (480, 240), 'KitDivider': (420, 270),
    'KitStat': (360, 210), 'KitChatBubble': (440, 230),
    'KitSkeleton': (420, 200), 'KitEmptyState': (480, 310),
    'KitTimeline': (480, 350), 'KitSteps': (420, 280),
    'KitAlert': (480, 220), 'KitRadialProgress': (320, 230),
    'KitTooltip': (360, 220), 'KitTooltipLayer': (360, 220),
    'KitToast': (480, 260), 'KitToastHost': (480, 390),
    'KitCollapse': (480, 250), 'KitAccordion': (520, 300),
    'KitCarousel': (480, 260), 'KitBreadcrumbs': (480, 160),
    'KitNotificationCenter': (620, 440), 'KitDrawer': (560, 420),
    'KitDataTable': (620, 400), 'KitContentList': (620, 410),
    'KitVirtualTextView': (620, 360), 'KitDialogSurface': (560, 360),
    'KitDialogPanel': (480, 260), 'KitDialogHeader': (420, 160),
    'KitDialogBody': (420, 180), 'KitDialogFooter': (420, 160),
})
bodies['KitOption'] = bodies['KitOption'].replace('selected: true;', 'selected: true; active: true;')
bodies['KitDrawer'] = bodies['KitDrawer'].replace('theme: root.theme;', 'theme: root.theme; panel-width: 380px * root.theme.text-scale;', 1).replace('wrap: word-wrap;', 'wrap: word-wrap; vertical-stretch: 0;')

missing=set(components)-set(bodies)
assert not missing, missing
for name,body in bodies.items():
    if name in ['KitCardHeader', 'KitRadioGroup', 'KitMultiSelect']:
        body = body.replace(name+' {', name+' { width: '+str(geometry[name][0] - 48)+'px;', 1)
    # Overlay primitives must be mounted directly at the window root.
    overlay=name in ['KitDrawer','KitDialogSurface']
    imports={'KitTheme','KitThemes'}|({name} if name!='KitTooltipLayer' else {'KitTooltipLayer'})
    imports.update(set(re.findall(r'\b[A-Z]\w+\b',body)) & public)
    tooltip = name in ['KitTooltip','KitTooltipLayer']
    if tooltip:imports.add('KitTooltipLayer')
    title=re.sub(r'(?<!^)(?=[A-Z])',' ',name.removeprefix('Kit'))
    header='// SPDX-License-Identifier: GPL-3.0-or-later\n'
    header+='// Display projections are fixtures. Validate accepted domain values in Rust.\n' if any(v in name for v in ['Money','Decimal','Date','Time','Percent','Quantity','Duration','Calendar','Month','Year','Week']) else ''
    source=header+'import { '+', '.join(sorted(imports))+' } from "@kit/kit.slint";\n\n'
    source+='export component Example inherits Window {\n'
    source+=f'    title: "{name} example";\n    preferred-width: {geometry[name][0]}px;\n    preferred-height: {geometry[name][1]}px;\n    max-width: 100000px;\n    max-height: 100000px;\n'
    source+='    background: root.theme.canvas;\n    in-out property <KitTheme> theme: KitThemes.light;\n    in-out property <string> status;\n'
    if extra[name]:source+='    '+extra[name]+'\n'
    if overlay:source+='    '+body+'\n'
    else:
        source+='    VerticalLayout {\n        padding: 24px; spacing: 16px; alignment: start;\n'
        source+='        Text { text: "'+title+'"; color: root.theme.text-primary; font-size: 20px * root.theme.text-scale; font-weight: 600; }\n'
        source+='        '+body+'\n'
        source+='        if root.status != "": Text { text: root.status; color: root.theme.text-secondary; font-size: 12px * root.theme.text-scale; wrap: word-wrap; }\n'
        source+='    }\n'
    if tooltip:source+='    KitTooltipLayer { viewport-width: root.width; viewport-height: root.height; }\n'
    source+='}\n'
    (ROOT/'docs/examples'/f'{slug(name)}.slint').write_text(source)
print(f'Generated {len(bodies)} public examples')

# Keep translations at the consumer boundary: import the shipped @kit unchanged.
# Toolkit/host strings without public overrides retain their original language.
import json
translation_file = ROOT/'docs/content/example-translations.json'
if translation_file.is_file():
    translations = json.loads(translation_file.read_text())
    target_directory = ROOT/'docs/examples/fr'
    target_directory.mkdir(parents=True, exist_ok=True)

    def closing_brace(source, opening):
        depth = 0
        quoted = False
        escaped = False
        comment = False
        for index in range(opening, len(source)):
            char = source[index]
            if comment:
                if char == '\n':
                    comment = False
                continue
            if quoted:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == '"':
                    quoted = False
                continue
            if source[index:index+2] == '//':
                comment = True
                continue
            if char == '"':
                quoted = True
            elif char == '{':
                depth += 1
            elif char == '}':
                depth -= 1
                if depth == 0:
                    return index
        raise ValueError('Unclosed Slint element')

    default_labels = {}
    for source_path in sorted(UI.glob('*.slint')):
        source = source_path.read_text()
        for declaration in re.finditer(r'export component (\w+) inherits (\w+)\s*\{', source):
            name = declaration[1]
            if name not in components:
                continue
            end = closing_brace(source, declaration.end()-1)
            body = source[declaration.end():end]
            defaults = {}
            for prop in re.finditer(r'in(?:-out)?\s+property\s*<string>\s*([\w-]+)\s*:\s*("(?:[^"\\]|\\.)*")', body):
                value = json.loads(prop[2])
                if value and value in translations:
                    defaults[prop[1]] = translations[value]
            default_labels[name] = defaults

    def inherited_labels(name):
        base = components.get(name)
        return {**(inherited_labels(base) if base in components else {}), **default_labels.get(name, {})}

    def direct_properties(body):
        # Nested callback/child/struct bodies cannot satisfy a parent's property.
        depth = 0
        quoted = False
        escaped = False
        output = []
        for char in body:
            if quoted:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == '"':
                    quoted = False
                output.append(' ')
                continue
            if char == '"':
                quoted = True
                output.append(' ')
            elif char == '{':
                depth += 1
                output.append(' ')
            elif char == '}':
                depth -= 1
                output.append(' ')
            else:
                output.append(char if depth == 0 else ' ')
        return set(re.findall(r'(?<![\w.-])([\w-]+)\s*:', ''.join(output)))

    for example in sorted((ROOT/'docs/examples').glob('*.slint')):
        source = example.read_text()
        additions = []
        for element in re.finditer(r'\b(Kit[A-Z]\w*)\s*\{', source):
            name = element[1]
            if name not in components:
                continue
            closing = closing_brace(source, element.end()-1)
            present = direct_properties(source[element.end():closing])
            labels = inherited_labels(name)
            assignments = ' '.join(prop+': '+json.dumps(value,ensure_ascii=False)+';' for prop,value in sorted(labels.items()) if prop not in present)
            if name in ['KitCalendar','KitDatePicker','KitDateRangePicker'] and 'weekdays' not in present:
                assignments += ' weekdays: ["L", "M", "M", "J", "V", "S", "D"];'
            if assignments:
                additions.append((element.end(), ' '+assignments))
        for position, addition in reversed(additions):
            source = source[:position]+addition+source[position:]
        def translate_literal(match):
            value = json.loads(match[0])
            return json.dumps(translations.get(value,value),ensure_ascii=False)
        source = re.sub(r'"(?:[^"\\]|\\.)*"', translate_literal, source)
        source = source.replace('["M", "T", "W", "T", "F", "S", "S"]','["L", "M", "M", "J", "V", "S", "D"]')
        source = source.replace('// SPDX-License-Identifier:', '// Public label overrides localize this fixture; toolkit/host defaults keep their language.\n// SPDX-License-Identifier:', 1)
        (target_directory/example.name).write_text(source)
    print(f'Generated {len(bodies)} localized French examples with public label overrides')
