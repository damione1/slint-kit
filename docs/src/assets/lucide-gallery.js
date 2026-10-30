// SPDX-License-Identifier: GPL-3.0-or-later
export function iconSnippet(name, mode, french = false) {
  if (!/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(name)) throw new Error('Invalid Lucide identifier');
  const label = french ? 'Ouvrir' : 'Open';
  let imports, content;
  switch (mode) {
    case 'standalone':
      imports='KitIcon';
      content=`KitIcon {\n        glyph: LucideIcons.${name}();\n    }`;
      break;
    case 'button':
      imports='KitIconButton';
      content=`KitIconButton {\n        glyph: LucideIcons.${name}();\n        accessible-name: "${label}";\n        activated => { root.action-requested(); }\n    }`;
      break;
    case 'text-button':
      imports='KitButton, ButtonIntent';
      content=`KitButton {\n        text: "${label}";\n        intent: ButtonIntent.primary;\n        leading-icon: LucideIcons.${name}();\n        activated => { root.action-requested(); }\n    }`;
      break;
    case 'pressable':
      imports='KitPressable, KitIcon, KitMetrics';
      content=`action := KitPressable {\n        accessible-name: "${label}";\n        preferred-width: 160px;\n        activated => { root.action-requested(); }\n        HorizontalLayout {\n            spacing: KitMetrics.inline-gap(action.theme);\n            alignment: center;\n            KitIcon {\n                theme: action.theme;\n                glyph: LucideIcons.${name}();\n                ink: action.disabled ? action.recipe.disabled-foreground : action.recipe.foreground;\n            }\n            Text {\n                text: "${label}";\n                color: action.disabled ? action.recipe.disabled-foreground : action.recipe.foreground;\n                font-family: action.theme.typography.family;\n                font-size: action.recipe.font-size;\n                font-weight: action.recipe.font-weight;\n                vertical-alignment: center;\n            }\n        }\n    }`;
      break;
    default: throw new Error(`Unknown icon integration: ${mode}`);
  }
  return `import { ${imports} } from "@kit/kit.slint";\nimport { LucideIcons } from "@kit/icons/lucide/all.slint";\n\nexport component App inherits Window {\n    preferred-width: 240px;\n    preferred-height: 100px;\n${mode==='standalone'?'':'    callback action-requested();\n'}    ${content}\n}`;
}

const gallery = typeof document === 'undefined' ? null : document.querySelector('.lucide-gallery');
if (gallery) {
  const french=document.documentElement.lang==='fr';
  const tiles=[...gallery.querySelectorAll('[data-icon]')];
  const search=gallery.querySelector('#icon-search'), category=gallery.querySelector('#icon-category');
  const mode=gallery.querySelector('#icon-integration'), snippet=gallery.querySelector('#icon-snippet');
  const name=gallery.querySelector('#selected-icon-name'), selected=gallery.querySelector('.selected-icon use');
  const help=gallery.querySelector('#icon-integration-help');
  let chosen=tiles.find(tile=>tile.dataset.icon==='search');
  const update=()=>{
    name.textContent=chosen.dataset.icon;
    selected.setAttribute('href',`${gallery.dataset.sprite}#${chosen.dataset.icon}`);
    snippet.textContent=iconSnippet(chosen.dataset.icon,mode.value,french);
    // Discard a previous icon's copy confirmation when the snippet changes.
    gallery.querySelectorAll('.code-actions [role=status]').forEach(status=>status.textContent='');
    help.textContent=mode.value==='standalone'
      ? (french?'Icône décorative. Pour porter un sens sans texte voisin, ajoutez IconPurpose.informative et un label.':'Decorative icon. To convey meaning without nearby text, add IconPurpose.informative and a label.')
      : (french?'Adaptez le nom accessible et branchez action-requested dans votre code hôte.':'Adapt the accessible name and connect action-requested in your host code.');
  };
  const filter=()=>{
    const terms=search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
    let count=0;
    for(const tile of tiles){
      tile.hidden=!(terms.every(term=>tile.dataset.search.includes(term))&&(!category.value||tile.dataset.categories.split(' ').includes(category.value)));
      if(!tile.hidden)count++;
    }
    gallery.querySelector('#icon-results').textContent=`${count} / ${tiles.length}`;
    gallery.querySelector('#icon-empty').hidden=count!==0;
  };
  search.addEventListener('input',filter);category.addEventListener('change',filter);
  mode.addEventListener('change',update);
  gallery.querySelector('#icon-grid').addEventListener('click',event=>{
    const tile=event.target.closest('[data-icon]');if(!tile)return;
    chosen.setAttribute('aria-pressed','false');chosen=tile;chosen.setAttribute('aria-pressed','true');update();
  });
  update();
}
