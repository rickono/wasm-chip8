import { Computer } from '../pkg/chip8';
import { memory } from '../pkg/chip8_bg';

const play = true;

const PIXEL_SIZE = 10;
const DIMS = [64, 32];

const canvas = document.getElementById('canvas-el');
canvas.width = DIMS[0] * PIXEL_SIZE;
canvas.height = DIMS[1] * PIXEL_SIZE;
const ctx = canvas.getContext('2d');

const registersDiv = document.getElementById('registers');
const instructionsDiv = document.getElementById('instructions');
const advanceButton = document.getElementById('next');
const advanceTenButton = document.getElementById('next-10');

let totalInst = 0;

const loadRom = async (computer) => {
  const rom = await fetch('./pong.ch8');
  const arrayBuffer = await rom.arrayBuffer();
  const data = new Uint8Array(arrayBuffer);
  const data16 = new DataView(arrayBuffer);
  for (let i = 0; i < data16.byteLength; i += 2) {
    const instruction = data16.getUint16(i);
    const instructionEl = document.createElement('p');
    const instructionIdx = document.createElement('p');
    instructionEl.innerHTML = instruction.toString(16).padStart(4, '0');
    instructionIdx.innerHTML = (i + 0x200).toString(16);
    const instructionDiv = document.createElement('div');
    instructionDiv.classList.add('instruction');
    instructionDiv.appendChild(instructionIdx);
    instructionDiv.appendChild(instructionEl);
    instructionsDiv.appendChild(instructionDiv);
  }
  computer.load(data);
};

const computer = Computer.new();
loadRom(computer);

advanceButton.addEventListener('click', () => {
  renderLoop();
});

advanceTenButton.addEventListener('click', () => {
  for (let i = 0; i < 10; i++) {
    renderLoop();
  }
});

document.addEventListener('keydown', (e) => {
  const key = parseInt(e.key, 16);
  if (key === 0 || key) {
    console.log(key);
    computer.keypress(parseInt(`0x${e.key}`));
  }
});

const renderLoop = () => {
  const inst = computer.tick();
  totalInst += 1;
  console.log(totalInst);
  if (inst > 0) {
    if ((inst & 0xf000) === 0xd000) {
      const pixelPtr = computer.pixels();
      const pixels = new BigUint64Array(memory.buffer, pixelPtr, 32);
      console.log(pixels);
    }
    console.log(inst.toString(16).padStart(4, '0'));
    console.log(computer.pc().toString(16).padStart(4, '0'));
  } else {
    console.log(new Uint8Array(memory.buffer, computer.memory(), 4096));
  }
  drawScreen();
  displayRegisters();
  if (play) {
    requestAnimationFrame(renderLoop);
  }
};

const displayRegisters = () => {
  registersDiv.innerHTML = '';
  const registerPtr = computer.registers();
  const registerContents = new Uint16Array(memory.buffer, registerPtr, 16);

  registerContents.forEach((register, idx) => {
    const contents = document.createElement('p');
    contents.innerHTML = register.toString(16).padStart(4, '0');
    registersDiv.appendChild(contents);
  });

  const i = computer.i();
  const contents = document.createElement('p');
  contents.innerHTML = i.toString(16).padStart(4, '0');
  registersDiv.appendChild(contents);
};

const drawScreen = () => {
  const pixelPtr = computer.pixels();
  const pixels = new BigUint64Array(memory.buffer, pixelPtr, 32);
  //   pixels is a 32-length array of 64-bit ints

  ctx.beginPath();

  for (const [y, row] of pixels.entries()) {
    const bits = row.toString(2).padStart(64, '0');
    for (const [x, pixel] of [...bits].entries()) {
      ctx.fillStyle = pixel === '0' ? 'red' : 'blue';

      ctx.fillRect(x * PIXEL_SIZE, y * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
    }
  }

  ctx.stroke();
};

drawScreen();
if (play) {
  requestAnimationFrame(renderLoop);
}
