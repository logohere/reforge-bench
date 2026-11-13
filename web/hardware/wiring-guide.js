import {element as node,commandButton,propertyList,actionLink} from '../ui/components.js';

import {wiringLegend} from './wire-conventions.js';

export const wiringSteps=[
  {
    "title": "Stop pulses; fit the wired TPH2",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": -1,
    "from": "Pico pin 1 at C23",
    "to": "TPH2 on Cora JA",
    "lead": "No jumper yet",
    "instruction": "Confirm GP2 is disabled or high impedance. Disconnect Cora power before fitting TPH2 to JA. Leave VCC unused; retain any existing harness. Use a rated passive scope probe.",
    "links": [
      [
        "Cora / TPH2 pin detail",
        "/hardware/#cora"
      ]
    ]
  },
  {
    "title": "Connect Pico ground to the shared row",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": 0,
    "from": "A25 · Pico GND / pin 3",
    "to": "B45 · shared ground row",
    "lead": "Black male–male jumper",
    "instruction": "Insert one metal pin at A25 and the other at B45. A45–E45 share this ground row. F45–J45 across the center gap are separate."
  },
  {
    "title": "Connect Pico pulse to the pulse row",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "pulse",
    "run": 0,
    "from": "A26 · Pico GP2 / pin 4",
    "to": "B55 · pulse row",
    "lead": "Yellow male–male jumper",
    "instruction": "Insert the jumper at A26 and B55. A55–E55 share the pulse signal; this row is separate from ground."
  },
  {
    "title": "Connect Cora common ground",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": 1,
    "from": "A45 · ground row",
    "to": "TPH2 GND · pin 5",
    "lead": "Black male–female jumper",
    "instruction": "With Cora off, connect A45 to TPH2 GND pin 5. Keep this jumper attached during programming. Do not join either board’s VCC. If already connected, check it and continue."
  },
  {
    "title": "Connect Cora pulse input",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "pulse",
    "run": 1,
    "from": "A55 · pulse row",
    "to": "TPH2 P1",
    "lead": "Yellow male–female jumper",
    "instruction": "With Cora off and Pico pulse output disabled, connect A55 to TPH2 P1. Keep it attached during programming; an existing correct connection can stay."
  },
  {
    "title": "Connect Cora return output",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "response",
    "run": 0,
    "from": "A60 · return row",
    "to": "TPH2 P2",
    "lead": "Blue male–female jumper",
    "instruction": "With Cora off, connect A60 to TPH2 P2. Keep it attached during programming; an existing correct connection can stay."
  },
  {
    "title": "Connect analyzer ground",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": 3,
    "from": "C45 · ground row",
    "to": "Small breakout · GND",
    "lead": "Black male–female jumper",
    "instruction": "Put the male end in C45 and the female end over a male post labelled GND on the fitted small breadboard adapter. Use its printed label; CLK is not ground. The two long rows duplicate the same signals."
  },
  {
    "title": "Connect analyzer channel 0",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "pulse",
    "run": 3,
    "from": "D55 · pulse row",
    "to": "Small breakout · 0 / CH0",
    "lead": "Yellow male–female jumper",
    "instruction": "Put the male end in D55 and the female end over the adapter male post labelled 0. This is a wiring step only. Do not start capture or enable the Pico."
  },
  {
    "title": "Connect analyzer channel 1",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "response",
    "run": 1,
    "from": "D60 · return row",
    "to": "Small breakout · 1 / CH1",
    "lead": "Blue male–female jumper",
    "instruction": "Put the male end in D60 and the female end over the adapter male post labelled 1. Leave CLK and channels 2–7 unused. No capture is taken yet."
  },
  {
    "title": "Connect the normal probe to scope CH1",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "pulse",
    "run": 4,
    "from": "Normal passive probe BNC",
    "to": "Scope CH1 BNC",
    "lead": "Normal 1×/10× passive probe",
    "instruction": "Connect and lock the normal probe BNC at CH1. Set its switch to 10×. Keep this probe on CH1 throughout this setup; do not use CH2, the P4100 or a BNC-to-clip lead."
  },
  {
    "title": "Connect the scope ground clip",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": 2,
    "from": "E45 · ground row",
    "to": "Normal probe’s short ground clip",
    "lead": "Black male–male jumper + probe clip",
    "instruction": "Insert a jumper in E45. Grip its free exposed metal pin with the normal probe’s short ground clip. This clip goes only to common ground, never a signal or VCC."
  },
  {
    "title": "Connect scope probe tip — power off",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "pulse",
    "run": 2,
    "from": "E55 · pulse row",
    "to": "Normal probe spring-hook tip",
    "lead": "Yellow male–male jumper + probe hook",
    "instruction": "Insert a jumper in E55 and attach the probe hook to its free exposed metal pin. Leave it attached. You are still wiring; you are not probing a live signal or taking a measurement."
  },
  {
    "title": "Wiring complete — STOP HERE",
    "phase": "Wiring · Cora OFF / Pico pulses OFF",
    "net": "ground",
    "run": -1,
    "from": "Breadboard, analyzer and CH1 probe wired",
    "to": "Keep all power off",
    "lead": "Visual connection check",
    "instruction": "Check each From → To pair and leave the completed harness connected. Cora is still off; Pico pulse output is disabled. VCC is unused. The normal probe stays on CH1, ground E45 and tip E55. Next opens checks; it does not switch power."
  },
  {
    "title": "Verify Pico output is disabled",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "usbPico",
    "run": 0,
    "from": "Acquisition-host USB cable",
    "to": "Pico USB",
    "lead": "USB data cable",
    "instruction": "Pico USB power is allowed to remain on. Inspect existing firmware/files and use the source-control tool to confirm GP2 output is disabled, input/high impedance or held low. USB power and pulse output are separate. Do not start the pulse."
  },
  {
    "title": "Connect analyzer USB to its host",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "usbAnalyzer",
    "run": 0,
    "from": "Acquisition-host USB cable",
    "to": "Analyzer USB",
    "lead": "Matching analyzer USB data cable",
    "instruction": "Connect the analyzer and confirm the host detects it. Keep the breadboard adapter attached. This step powers/connects the analyzer; it does not start acquisition."
  },
  {
    "title": "Reload and verify Cora over USB",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "usbCora",
    "run": 0,
    "from": "Programming-host USB cable",
    "to": "Cora J12 USB",
    "lead": "Wired, stimulus-disabled guarded reload",
    "instruction": "Keep the fitted TPH2 and all jumper wires connected. With Pico GP2 confirmed disabled, common ground present and VCC supplies isolated, connect Cora USB. Run the guarded loader in wired-stimulus-disabled mode. Verify target, image hash, DONE and configuration errors. Keep Cora powered afterward.",
    "links": [
      [
        "Wired programming procedure",
        "/hardware/cora-wired-bringup.md"
      ]
    ]
  },
  {
    "title": "Check Cora ground connection",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "ground",
    "run": 1,
    "from": "A45 · common ground",
    "to": "TPH2 GND · pin 5",
    "lead": "Prepared black male–female lead",
    "instruction": "Check the existing black jumper: A45 → TPH2 GND pin 5. Leave it connected. Keep both boards’ VCC pins unconnected."
  },
  {
    "title": "Check Cora P1 input",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "pulse",
    "run": 1,
    "from": "A55 · pulse row",
    "to": "TPH2 P1",
    "lead": "Prepared yellow male–female lead",
    "instruction": "Check the existing yellow jumper: A55 → TPH2 P1, pulse input. Keep Pico output disabled and leave the connection in place."
  },
  {
    "title": "Check Cora P2 return",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "response",
    "run": 0,
    "from": "A60 · return row",
    "to": "TPH2 P2",
    "lead": "Prepared blue male–female lead",
    "instruction": "Check the existing blue jumper: TPH2 P2 → A60, return output. Analyzer channel 1 reads this net. Leave it connected."
  },
  {
    "title": "Connect and configure scope CH1",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "usbScope",
    "run": 0,
    "from": "Scope rear USB-B",
    "to": "Configured USB acquisition adapter",
    "lead": "Scope mains and USB data cables",
    "instruction": "Turn on the scope with its grounded mains supply. Match CH1 Probe to 10× and use DC coupling. Keep the normal probe on CH1. Connect rear USB-B to the acquisition host; front USB-A is not the PC link."
  },
  {
    "title": "Verify the Acquisition adapter connection",
    "phase": "Power and verify · PULSE OUTPUT OFF",
    "net": "usbScope",
    "run": 0,
    "from": "Configured acquisition sender",
    "to": "Configured capture receiver",
    "lead": "User-configured acquisition bridge",
    "instruction": "Use your acquisition tool or configured adapter. Confirm the source identity, fresh hardware frames and matching raw hash before recording. Keep stimulus output disabled.",
    "links": [
      [
        "Open Acquire",
        "/hardware/#acquisition"
      ],
      [
        "Open Live bench",
        "/hardware/#live"
      ]
    ]
  },
  {
    "title": "Enable pulses, record, then stop",
    "phase": "Measure · enable only after checks",
    "net": "pulse",
    "run": -1,
    "from": "Pico GP2 → Cora P1",
    "to": "Analyzer 0 / 1 and scope CH1",
    "lead": "Your acquisition tool",
    "instruction": "After wiring and supply checks, enable the example 1 kHz source. Record both digital channels and the scope pulse. Disable the source afterward and retain original captures, settings and identities.",
    "links": [
      [
        "Open Acquire",
        "/hardware/#acquisition"
      ],
      [
        "Open Live bench",
        "/hardware/#live"
      ]
    ]
  }
];

export const ddcsWiringSteps=[
  {title:'Power off before adding UART',phase:'Wiring · Pico and Cora USB OFF',net:'ground',run:-1,from:'Existing pulse-test harness',to:'Pico and Cora USB disconnected',lead:'No new wire yet',instruction:'Keep all existing jumpers, the analyzer and the normal CH1 probe in place. Disconnect Pico and Cora USB power before adding the two UART jumpers. Match printed P3 and P4 labels, not connector viewing orientation. VCC posts stay empty.'},
  {title:'Keep the common ground',phase:'Wiring · Pico and Cora USB OFF',net:'ground',run:1,from:'A45 · existing common ground row',to:'TPH2 GND · pin 5',lead:'Existing black male–female jumper',instruction:'Keep A45 → TPH2 GND pin 5 unchanged. Pico GND, analyzer GND and the CH1 probe ground still share A45–E45. Do not join supply outputs.'},
  {title:'Keep STEP observation wiring',phase:'Wiring · Pico and Cora USB OFF',net:'pulse',run:1,from:'A55 · existing signal row',to:'TPH2 P1 · JA1 / Y18',lead:'Existing yellow male–female jumper',instruction:'Keep A55 → TPH2 P1 unchanged. GP2 / A26, analyzer CH0 / D55 and scope CH1 / E55 stay on this same net. In the DDCS profile, FPGA P1 drives STEP and Pico GP2 must be input only.'},
  {title:'Keep DIR observation wiring',phase:'Wiring · Pico and Cora USB OFF',net:'response',run:0,from:'TPH2 P2 · JA2 / Y19',to:'A60 · existing signal row',lead:'Existing blue male–female jumper',instruction:'Keep TPH2 P2 → A60 and analyzer CH1 / D60 unchanged. This net carries DIR in the DDCS profile, not the original pulse-loop heartbeat.'},
  {title:'Add Pico TX to FPGA RX',phase:'Wiring · Pico and Cora USB OFF',net:'uartTx',run:0,from:'A23 · Pico GP0 / physical pin 1 at C23',to:'TPH2 P3 · JA3 / Y16 · FPGA RX',lead:'New green male–female jumper',instruction:'Put the male end in free hole A23, which shares the Pico pin-1 strip C23. Put the female end over the upright post printed P3 on TPH2. This is Pico UART TX to FPGA UART RX. Keep power off.'},
  {title:'Add FPGA TX to Pico RX',phase:'Wiring · Pico and Cora USB OFF',net:'uartRx',run:0,from:'A24 · Pico GP1 / physical pin 2 at C24',to:'TPH2 P4 · JA4 / Y17 · FPGA TX',lead:'New white male–female jumper',instruction:'Put the male end in free hole A24, which shares the Pico pin-2 strip C24. Put the female end over the upright post printed P4 on TPH2. This connects FPGA UART TX to Pico UART RX. Do not use VCC, P7 or P10.'},
  {title:'Wiring complete — STOP HERE',phase:'Wiring · Pico and Cora USB OFF',net:'uartTx',run:null,from:'Two additional UART jumpers fitted',to:'Keep both boards powered off',lead:'Visual connection check; continuity meter optional',instruction:'Check A23 → P3 and A24 → P4. Existing rows 45, 55 and 60 stay unchanged. Selecting a guide step does not verify physical wiring, change profiles or enable output. Confirm the fitted wires with the test operator before continuing.'},
  {title:'Verify Pico UART profile — pending',phase:'Power and verify · PENDING',net:'usbPico',run:0,from:'Acquisition-host USB cable',to:'Pico USB',lead:'Reviewed Pico profile installer',instruction:'After the wiring check, reconnect Pico USB. Verify the DDCS UART runtime, GP0 TX / GP1 RX at 115200 baud and GP2 input / high impedance. Pico PWM must remain disabled. This guide does not install firmware or claim the runtime was checked.',links:[['Open DDCS proof','/hardware/#ddcs']]},
  {title:'Load qualified FPGA profile — pending',phase:'Power and verify · PENDING',net:'usbCora',run:0,from:'Programming-host USB / JTAG',to:'Cora J12 · qualified DDCS image',lead:'Guarded volatile programming; no flash write',instruction:'Only after GP2 is confirmed input, reconnect Cora USB and load the reviewed DDCS STEP/DIR image. Verify exact target, image hash and configuration status. Power loss clears the volatile image. Do not restore Pico PWM while this image drives P1.'},
  {title:'Check UART identity — pending',phase:'Power and verify · PENDING',net:'uartTx',run:null,from:'Pico GP0 → FPGA P3 RX',to:'FPGA P4 TX → Pico GP1',lead:'DDCS proof → Check board',instruction:'Check the actual board design and session over UART before submitting a job. Missing or mismatched replies are a failed check, not permission to retry a motion command. The drawn diagram alone is not physical verification.',links:[['Open DDCS proof','/hardware/#ddcs']]},
  {title:'Capture finite DDCS job — pending',phase:'Measure · PENDING',net:'pulse',run:-1,from:'FPGA STEP / DIR on existing signal rows',to:'Analyzer CH0 / CH1 and scope CH1',lead:'DDCS proof → original / modified captures',instruction:'Use the DDCS proof controls to compare firmware and capture one bounded job. Verify signed pulse count, timing and direction, then retain raw evidence. For rollback, restore the original FPGA input image before restoring the Pico pulse profile. These steps are pending until actual receipts are retained.',links:[['Open DDCS proof','/hardware/#ddcs']]},
];

export const wiringStepsFor=profile=>profile==='ddcs'?ddcsWiringSteps:wiringSteps;

export function wiringGuide({initialStep=0,onChange=()=>{},profile='pulse'}={}){
  const root=node('section',undefined,'hardware-wire-guide'),heading=node('div',undefined,'hardware-guide-heading'),count=node('span'),phase=node('p',undefined,'hardware-guide-phase'),route=node('div',undefined,'hardware-guide-route'),title=node('h2'),instruction=node('ol',undefined,'hardware-guide-instructions'),properties=node('div'),controls=node('div',undefined,'hardware-guide-actions'),back=commandButton('Back',{kind:'ghost'}),next=commandButton('Next',{kind:'primary'});let index=initialStep;
  let steps=wiringStepsFor(profile);const legend=node('div');legend.append(wiringLegend(profile));
  const body=node('div',undefined,'hardware-guide-body');instruction.setAttribute('aria-label','Wiring instruction');body.tabIndex=0;body.setAttribute('aria-label','Wiring step instructions');root.setAttribute('aria-label','Step-by-step wiring');heading.append(count,title);controls.append(back,next);body.append(route,instruction,properties,legend);root.append(phase,heading,body,controls);
  function setStep(value){if(!Number.isInteger(value)||value<0||value>=steps.length)throw new Error('Unknown wiring step');index=value;const step=steps[index];root.dataset.step=String(index);root.dataset.net=step.net;const phaseSteps=steps.filter(item=>item.phase===step.phase),phaseIndex=phaseSteps.indexOf(step)+1;count.textContent=phaseIndex+' / '+phaseSteps.length;title.textContent=step.title;phase.textContent=step.phase;phase.dataset.stage=step.phase.startsWith('Wiring')?'wire':step.phase.startsWith('Power')?'power':'measure';route.replaceChildren();for(const [label,value] of [['FROM',step.from],['TO',step.to]]){const endpoint=node('div',undefined,'hardware-guide-endpoint');endpoint.append(node('span',label),node('strong',value));route.append(endpoint);}route.dataset.net=step.net;instruction.replaceChildren(...step.instruction.split(/(?<=[.!?])\s+(?=[A-Z])/).map(text=>{const li=node('li');for(const fragment of text.split(/(\b[A-J]\d{1,2}\b)/g))li.append(node(/^[A-J]\d{1,2}$/.test(fragment)?'code':'span',fragment));return li;}));body.scrollTop=instruction.scrollTop=0;properties.replaceChildren(propertyList([['USE THIS LEAD',step.lead]]));for(const [label,href] of step.links||[])properties.append(actionLink(label,href));back.disabled=index===0;next.disabled=index===steps.length-1;next.textContent=step.title.includes('STOP HERE')?'Power checklist':step.phase.startsWith('Wiring')?'Next wiring step':step.phase.startsWith('Power')?'Next check':'Complete';onChange(step,index);}
  back.addEventListener('click',()=>setStep(index-1));next.addEventListener('click',()=>setStep(index+1));
  return {root,setStep,setProfile(value){profile=value;steps=wiringStepsFor(profile);legend.replaceChildren(wiringLegend(profile));index=0;},advance(){if(index<steps.length-1)setStep(index+1);},get step(){return index;}};
}
