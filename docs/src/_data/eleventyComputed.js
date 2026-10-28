// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import { pageMetadata } from '../../scripts/metadata.mjs';
const pages = JSON.parse(fs.readFileSync(new URL('../../content/pages.json', import.meta.url), 'utf8'));
export default {
  seo: data => data.translationKey !== undefined || data.entry ? pageMetadata(data, pages) : undefined,
};
