// SPDX-License-Identifier: GPL-3.0-or-later
import { localeUrl } from './routes.mjs';

export function pageMetadata(data, pages) {
  const component = data.entry?.component;
  const locale = data.entry?.locale || data.locale;
  const key = component ? `components/${component.slug}/` : data.translationKey;
  const copy = component ? null : pages[key]?.[locale];
  if (!component && !copy) throw new Error(`Missing page metadata: ${locale}/${key}`);
  const title = component ? `${component.name} · Slint Kit` : copy.title;
  const description = component ? component[locale].summary : copy.description.replaceAll('{count}', data.components.length);
  const url = data.site.url + localeUrl(key, locale);
  const breadcrumbs = key ? [{ name: locale === 'fr' ? 'Accueil' : 'Home', url: data.site.url + localeUrl('', locale) }] : [];
  if (component) breadcrumbs.push({ name: locale === 'fr' ? 'Composants' : 'Components', url: data.site.url + localeUrl('components/', locale) });
  if (key) breadcrumbs.push({ name: component?.name || copy.label, url });
  return {
    title, description, url, breadcrumbs,
    pageType: !key ? 'WebPage' : key === 'components/' ? 'CollectionPage' : 'TechArticle',
    image: `${data.site.url}/assets/social/slint-kit-${locale}.png`,
    imageAlt: locale === 'fr' ? 'Slint Kit — Composants UI natifs pour Rust et Slint' : 'Slint Kit — Native UI components for Rust and Slint',
    locale: locale === 'fr' ? 'fr_CA' : 'en_CA',
    alternateLocale: locale === 'fr' ? 'en_CA' : 'fr_CA',
  };
}

export function structuredData(seo, site, locale) {
  const sourceId = `${site.url}/#source`;
  const websiteId = `${site.url}/#website`;
  const page = {
    '@type': seo.pageType, '@id': `${seo.url}#webpage`,
    url: seo.url, name: seo.title, description: seo.description, inLanguage: locale,
    isPartOf: { '@id': websiteId }, about: { '@id': sourceId },
  };
  const graph = [
    { '@type': 'WebSite', '@id': websiteId, url: `${site.url}/`, name: site.name, inLanguage: ['en', 'fr'] },
    { '@type': 'SoftwareSourceCode', '@id': sourceId, name: site.name, url: `${site.url}/`, codeRepository: site.repository, programmingLanguage: ['Rust', 'Slint'], license: `${site.repository}/blob/main/LICENSE` },
    page,
  ];
  if (seo.breadcrumbs.length) {
    page.breadcrumb = { '@id': `${seo.url}#breadcrumb` };
    graph.push({ '@type': 'BreadcrumbList', '@id': `${seo.url}#breadcrumb`, itemListElement: seo.breadcrumbs.map((crumb, index) => ({ '@type': 'ListItem', position: index + 1, name: crumb.name, item: crumb.url })) });
  }
  return { '@context': 'https://schema.org', '@graph': graph };
}

// JSON lives in a script raw-text element: HTML escaping is insufficient here.
export const scriptJson = value => JSON.stringify(value).replace(/[<>&\u2028\u2029]/g, character => `\\u${character.charCodeAt(0).toString(16).padStart(4, '0')}`);
