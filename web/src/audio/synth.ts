/**
 * Tiny Web Audio synthesizer: every game sound is built from tones and
 * filtered noise at play time, so there are no audio files to load or license.
 */

/** Loudness of every sound together; individual sounds stay below 1. */
const MASTER_GAIN = 0.25

let context: AudioContext | null = null
let master: GainNode | null = null

/**
 * The shared audio context, created on first use. Browsers keep it suspended
 * until the player interacts with the page, so it resumes on the first key or
 * pointer press.
 */
function audio(): { ctx: AudioContext; out: GainNode } | null {
  if (typeof AudioContext === 'undefined') return null
  if (context === null || master === null) {
    context = new AudioContext()
    master = context.createGain()
    master.gain.value = MASTER_GAIN
    master.connect(context.destination)
    const resume = () => void context?.resume()
    window.addEventListener('pointerdown', resume, { once: true })
    window.addEventListener('keydown', resume, { once: true })
  }
  return { ctx: context, out: master }
}

/** An oscillator note. Times in seconds from now. */
export interface Tone {
  wave: OscillatorType
  from: number
  /** Pitch at the end (a slide); `from` when absent. */
  to?: number
  start?: number
  duration: number
  volume: number
}

/** A burst of white noise through a filter. Times in seconds from now. */
interface Noise {
  filter: BiquadFilterType
  frequency: number
  /** Filter frequency at the end (a sweep). */
  sweepTo?: number
  start?: number
  duration: number
  volume: number
}

/** A sound: tones and noise bursts played together. */
export interface Sound {
  tones?: Tone[]
  noise?: Noise[]
}

/** Gain that jumps to `volume` and decays to silence over `duration`. */
function envelope(ctx: AudioContext, at: number, duration: number, volume: number): GainNode {
  const gain = ctx.createGain()
  gain.gain.setValueAtTime(0.0001, at)
  gain.gain.exponentialRampToValueAtTime(volume, at + 0.005)
  gain.gain.exponentialRampToValueAtTime(0.0001, at + duration)
  return gain
}

let noiseBuffer: AudioBuffer | null = null

function whiteNoise(ctx: AudioContext): AudioBuffer {
  if (noiseBuffer === null) {
    noiseBuffer = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate)
    const data = noiseBuffer.getChannelData(0)
    for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1
  }
  return noiseBuffer
}

/** Plays a sound now; silently does nothing where Web Audio is missing. */
export function play(sound: Sound): void {
  const target = audio()
  if (target === null) return
  const { ctx, out } = target
  const now = ctx.currentTime

  for (const tone of sound.tones ?? []) {
    const at = now + (tone.start ?? 0)
    const osc = ctx.createOscillator()
    osc.type = tone.wave
    osc.frequency.setValueAtTime(tone.from, at)
    if (tone.to !== undefined)
      osc.frequency.exponentialRampToValueAtTime(tone.to, at + tone.duration)
    osc.connect(envelope(ctx, at, tone.duration, tone.volume)).connect(out)
    osc.start(at)
    osc.stop(at + tone.duration + 0.02)
  }

  for (const burst of sound.noise ?? []) {
    const at = now + (burst.start ?? 0)
    const source = ctx.createBufferSource()
    source.buffer = whiteNoise(ctx)
    const filter = ctx.createBiquadFilter()
    filter.type = burst.filter
    filter.frequency.setValueAtTime(burst.frequency, at)
    if (burst.sweepTo !== undefined) {
      filter.frequency.exponentialRampToValueAtTime(burst.sweepTo, at + burst.duration)
    }
    source
      .connect(filter)
      .connect(envelope(ctx, at, burst.duration, burst.volume))
      .connect(out)
    source.start(at)
    source.stop(at + burst.duration + 0.02)
  }
}
