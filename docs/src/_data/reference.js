// SPDX-License-Identifier: GPL-3.0-or-later
import { scanComponents } from '../../scripts/inventory.mjs';
export default () => scanComponents().flatMap(component => ['fr', 'en'].map(locale => ({ locale, component })));
