import type { Sound, Tone } from './synth.ts'

/** Notes one after another, `step` seconds apart. */
function arpeggio(notes: number[], step: number, wave: OscillatorType, volume: number): Tone[] {
  return notes.map((from, i) => ({ wave, from, start: i * step, duration: step * 2.5, volume }))
}

/** Every sound the battle makes, by name. */
export const SOUNDS = {
  hit: {
    tones: [{ wave: 'sine', from: 140, to: 50, duration: 0.12, volume: 0.9 }],
    noise: [{ filter: 'lowpass', frequency: 1400, duration: 0.08, volume: 0.6 }],
  },
  critical: {
    tones: [
      { wave: 'sine', from: 160, to: 40, duration: 0.2, volume: 1 },
      { wave: 'square', from: 1200, to: 500, duration: 0.07, volume: 0.25 },
    ],
    noise: [{ filter: 'lowpass', frequency: 2200, duration: 0.14, volume: 0.8 }],
  },
  miss: {
    noise: [{ filter: 'bandpass', frequency: 2400, sweepTo: 500, duration: 0.18, volume: 0.35 }],
  },
  fire: {
    noise: [
      { filter: 'lowpass', frequency: 3000, sweepTo: 300, duration: 0.45, volume: 0.8 },
      { filter: 'bandpass', frequency: 900, start: 0.05, duration: 0.3, volume: 0.5 },
    ],
  },
  ice: {
    tones: arpeggio([2093, 2637, 3136, 2349], 0.04, 'triangle', 0.35),
    noise: [{ filter: 'highpass', frequency: 5000, duration: 0.25, volume: 0.25 }],
  },
  lightning: {
    tones: [{ wave: 'sawtooth', from: 70, to: 45, duration: 0.3, volume: 0.35 }],
    noise: [{ filter: 'highpass', frequency: 1800, duration: 0.09, volume: 0.9 }],
  },
  gravity: {
    tones: [
      { wave: 'sine', from: 220, to: 35, duration: 0.6, volume: 0.9 },
      { wave: 'triangle', from: 110, to: 30, duration: 0.6, volume: 0.4 },
    ],
  },
  poison: {
    tones: [
      { wave: 'sine', from: 380, to: 220, duration: 0.09, volume: 0.4 },
      { wave: 'sine', from: 320, to: 180, start: 0.1, duration: 0.09, volume: 0.35 },
    ],
  },
  heal: { tones: arpeggio([523, 659, 784, 1047], 0.07, 'triangle', 0.4) },
  revive: { tones: arpeggio([392, 523, 659, 784, 1047, 1319], 0.08, 'triangle', 0.4) },
  cure: { tones: arpeggio([1319, 1760], 0.06, 'triangle', 0.35) },
  status: {
    tones: [
      { wave: 'square', from: 520, to: 300, duration: 0.22, volume: 0.15 },
      { wave: 'square', from: 530, to: 310, duration: 0.22, volume: 0.12 },
    ],
  },
  defend: {
    tones: [{ wave: 'square', from: 240, to: 180, duration: 0.08, volume: 0.2 }],
    noise: [{ filter: 'highpass', frequency: 3500, duration: 0.12, volume: 0.35 }],
  },
  item: { tones: arpeggio([880, 1320], 0.05, 'triangle', 0.35) },
  knockout: {
    tones: [{ wave: 'sine', from: 240, to: 45, duration: 0.5, volume: 0.8 }],
    noise: [{ filter: 'lowpass', frequency: 600, duration: 0.3, volume: 0.4 }],
  },
  turn: { tones: arpeggio([660, 990], 0.06, 'triangle', 0.3) },
  victory: { tones: arpeggio([523, 659, 784, 1047, 784, 1047], 0.11, 'square', 0.18) },
  defeat: { tones: arpeggio([440, 349, 294, 220], 0.18, 'triangle', 0.4) },
} satisfies Record<string, Sound>

/** Name of a battle sound. */
export type SoundName = keyof typeof SOUNDS
