#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check that the design review accounts for every documented component."""
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
catalogue = json.loads((root / 'docs/content/components.json').read_text())
reviewed = set()
for path in sorted((root / 'design').glob('review-*.md')):
    names = set(re.findall(r'`(Kit\w+)`', path.read_text())) & catalogue.keys()
    reviewed.update(names)
    print(f'{path.name}: {len(names)} public components')
missing = sorted(catalogue.keys() - reviewed)
if missing:
    raise SystemExit('Unreviewed components: ' + ', '.join(missing))
print(f'PASS: all {len(catalogue)} documented public components are accounted for.')
