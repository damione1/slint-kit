// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
// Remove generated pages from previous routes before building or serving.
fs.rmSync(fileURLToPath(new URL('../_site/', import.meta.url)), { recursive: true, force: true });
