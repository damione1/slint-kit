# SPDX-License-Identifier: GPL-3.0-or-later
"""The single built-in presentation used by documentation consumers."""
from enum import Enum
import re


class ThemeFamily(Enum):
    TAILWIND = 'tailwind'


def themed_source(source, family):
    ThemeFamily(family)
    # Downloaded examples already choose Inter. Keep this compatibility helper
    # idempotent for legacy consumer sources while using the built-in theme.
    if 'InterFont' not in source:
        source = 'import { InterFont } from "@kit/fonts/inter.slint";\n' + source
    if 'default-font-family:' not in source:
        source, count = re.subn(r'(export\s+component\s+\w+\s+inherits\s+Window\s*\{)',
                               r'\1\n    default-font-family: InterFont.family;', source, count=1)
        if count != 1:
            raise ValueError('A preview must expose a Window')
    return source
