import {element,modalDialog} from '../ui/components.js';

// Reuse the live viewport so detail and bench never diverge into separate windows.
export function partDialog({parent,toolbar,viewport,selection,status,onClose}){
  let disposed=false;
  const dialog=modalDialog({title:'Component detail'}),body=element('div',undefined,'hardware-inspection-grid');
  dialog.root.classList.add('hardware-part-dialog');parent.append(dialog.root);
  const closeButton=dialog.root.querySelector('.ui-dialog-header>cds-button');
  closeButton.textContent='×';closeButton.setAttribute('aria-label','Close component detail');closeButton.title='Close · Escape';
  dialog.root.addEventListener('close',()=>{if(disposed)return;parent.prepend(toolbar);parent.insertBefore(viewport,dialog.root);parent.insertBefore(selection,dialog.root);parent.insertBefore(status,dialog.root);onClose();});
  return {show(title){if(dialog.root.open){dialog.heading.textContent=title;return;}body.append(toolbar,viewport,selection,status);dialog.show(title,body);},close(){if(dialog.root.open)dialog.close();},dispose(){disposed=true;if(dialog.root.open)dialog.close();dialog.root.remove();}};
}
