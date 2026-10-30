// SPDX-License-Identifier: GPL-3.0-or-later
// Original geometric scenes. These illustrate a family, not native screenshots.
const rect = (x,y,w,h,fill='paper',radius=5) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${radius}" class="family-${fill}"/>`;
const line = (x,y,w,fill='line',h=5) => rect(x,y,w,h,fill,2);
const circle = (x,y,r,fill='line') => `<circle cx="${x}" cy="${y}" r="${r}" class="family-${fill}"/>`;
const path = (d,fill='line') => `<path d="${d}" class="family-${fill}"/>`;
const check = (x,y) => `<path d="m${x} ${y} 4 4 8-9" fill="none" class="family-check" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>`;
const rows = (x,y,w,count=3) => Array.from({length:count},(_,i)=>line(x,y+i*24,w-i%2*20)).join('');
const scenes = {
  button: () => rect(58,75,145,46,'accent',7)+rect(215,75,88,46)+line(82,95,96,'paper')+line(232,95,53),
  field: () => rect(65,29,230,142)+line(84,48,73)+rect(84,62,191,34,'canvas')+line(96,77,98)+line(84,107,154,'soft',4)+line(84,123,119,'soft',4)+line(84,141,62,'accent',4),
  input: () => rect(54,32,252,136)+line(72,51,58,'line',4)+rect(72,64,216,33,'canvas')+line(84,78,106)+line(72,110,42,'line',4)+rect(72,122,216,28,'canvas')+Array.from({length:6},(_,i)=>circle(87+i*10,136,2.5)).join(''),
  number: () => rect(70,48,220,105)+line(89,68,55)+rect(89,83,182,47,'canvas')+line(104,103,65,'accent',7)+path('M240 98h17l-8.5-8.5z')+path('M240 112h17l-8.5 8.5z'),
  select: () => rect(74,30,212,35)+line(92,45,108)+path('m253 43 7 7 7-7z')+rect(74,74,212,101)+rect(81,106,198,27,'soft')+rows(93,89,125)+check(250,118),
  choice: () => rect(64,43,232,114)+rect(83,62,17,17,'accent',4)+check(86,69)+line(113,68,115)+circle(91.5,101,8,'soft')+circle(91.5,101,3.5,'accent')+line(113,98,84)+rect(82,123,35,19,'accent',10)+circle(107,132.5,6.5,'paper')+line(131,130,96),
  calendar: () => rect(87,23,186,154)+line(119,43,80)+path('m107 40-5 5 5 5z')+path('m253 40 5 5-5 5z')+Array.from({length:28},(_,i)=>rect(102+(i%7)*23,68+Math.floor(i/7)*24,15,15,i===17?'accent':'soft',3)).join(''),
  datetime: () => rect(65,51,135,97)+rows(82,74,97,2)+rect(82,121,55,12,'soft')+circle(245,98,47,'paper')+`<path d="M245 69v29l20 13" fill="none" class="family-clock" stroke-width="5" stroke-linecap="round"/>`+circle(245,98,4,'accent'),
  files: () => path('M65 66V48q0-7 7-7h68l17 19h116q8 0 8 8v84H65z','soft')+rect(65,76,216,86)+path('M184 58h49l21 22v65h-70z')+path('M233 58v23h21z','soft')+rows(196,99,43,2),
  color: () => rect(78,31,204,140)+Array.from({length:12},(_,i)=>`<rect x="${96+i%4*44}" y="${49+Math.floor(i/4)*37}" width="34" height="27" rx="4" fill="hsl(${210+(i%4)*14} ${12+Math.floor(i/4)*17}% ${78-Math.floor(i/4)*18}%)"/>`).join('')+check(191,98),
  layout: () => rect(49,37,262,126)+rect(62,50,236,20,'soft')+rect(62,81,64,69,'line')+rect(137,81,72,69,'soft')+rect(220,81,78,30,'soft')+rect(220,122,78,28,'soft'),
  scroll: () => rect(79,27,202,146)+rows(96,46,137,5)+rect(263,38,6,124,'canvas',3)+rect(263,65,6,47,'accent',3),
  display: () => rect(52,51,122,98)+rect(186,51,122,98)+circle(78,76,10,'soft')+line(96,73,56)+line(68,105,78,'accent',11)+line(68,127,64,'soft',4)+rect(204,93,15,40,'soft')+rect(229,77,15,56,'line')+rect(254,66,15,67,'accent'),
  feedback: () => rect(61,35,238,40)+circle(83,55,9,'accent')+line(104,47,138,'line',4)+line(104,59,98,'soft',4)+rect(61,88,238,40)+circle(83,108,9,'soft')+line(104,101,119,'line',4)+line(104,113,153,'soft',4)+rect(89,145,182,8,'paper',4)+rect(89,145,116,8,'accent',4),
  navigation: () => rect(54,35,252,132)+rect(67,49,226,22,'canvas')+rect(72,53,54,14,'accent',3)+line(139,58,42,'line',4)+line(197,58,46,'line',4)+rect(67,84,54,69,'soft')+rows(135,90,132,3),
  dialog: () => rect(45,28,270,145,'soft')+rect(91,49,178,112)+line(108,67,99)+line(108,85,143,'soft',4)+line(108,97,119,'soft',4)+rect(194,123,57,23,'accent',4)+rect(140,123,44,23,'canvas',4),
  data: () => rect(42,38,276,124)+rect(42,38,276,25,'canvas')+line(57,48,64,'line',4)+line(162,48,43,'line',4)+line(247,48,42,'line',4)+[79,108,137].map((y,i)=>(i===1?rect(48,y-9,264,24,'soft',3):'')+rect(57,y-3,9,9,'line',2)+line(75,y,60+i%2*15,'line',4)+line(162,y,40,'line',4)+rect(247,y-4,43,12,'soft',6)).join(''),
  focus: () => `<rect x="81" y="57" width="198" height="86" rx="12" fill="none" class="family-clock" stroke-width="3" stroke-dasharray="7 5"/>`+rect(91,67,178,66)+line(121,95,118,'accent',8),
  icons: () => rect(53,62,72,76)+rect(144,62,72,76)+rect(235,62,72,76)+`<path d="m73 100 11 11 20-25M166 88l26 26m0-26-26 26M270 86v28m-14-14h28" fill="none" class="family-clock" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>`,
};
export function familyIllustration(family) {
  const scene = scenes[family];
  if (!scene) throw new Error(`Missing original illustration for ${family}`);
  return `<svg class="family-illustration" viewBox="0 0 360 200" width="360" height="200" aria-hidden="true" focusable="false"><defs><filter id="shadow-${family}" x="-30%" y="-30%" width="160%" height="180%"><feDropShadow dx="0" dy="5" stdDeviation="5" flood-color="currentColor" flood-opacity=".1"/></filter></defs>${rect(0,0,360,200,'canvas',12)}<g filter="url(#shadow-${family})">${scene()}</g></svg>`;
}
