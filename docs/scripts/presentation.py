# SPDX-License-Identifier: GPL-3.0-or-later
"""Presentation variants of the same runnable starter, without changing its host."""
from enum import Enum
import re


class ThemeFamily(Enum):
    KIT = 'kit'
    TAILWIND = 'tailwind'


def themed_source(source, family):
    family = ThemeFamily(family)
    if family is ThemeFamily.KIT:
        return source
    # Keep the downloadable application independent of this optional theme.
    # All theme literals and implicit component defaults agree in this window.
    imports = 'import { TailwindThemes } from "@kit/themes/tailwind.slint";\n'
    if not re.search(r'\bKitThemeContext\b', source):
        imports += 'import { KitThemeContext } from "@kit/kit.slint";\n'
    source = source.replace('KitThemes.light', 'TailwindThemes.light')
    source, count = re.subn(r'(export\s+component\s+\w+\s+inherits\s+Window\s*\{)',
                           r'\1\n    init => { KitThemeContext.current = TailwindThemes.light; }', source, count=1)
    if count != 1:
        raise ValueError('A preview must expose a Window')
    return imports + source
