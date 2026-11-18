import { describe, expect, test } from 'vitest'

import { classNames } from './class-names.ts'
import { initialsOf } from './initials.ts'
import { percentOf } from './percent.ts'

describe('classNames', () => {
  test('joins truthy names and skips the rest', () => {
    expect(classNames('a', false, 'b', null, undefined, '')).toBe('a b')
  })
})

describe('percentOf', () => {
  test('clamps to 0–100 and guards a zero max', () => {
    expect(percentOf(30, 120)).toBe(25)
    expect(percentOf(150, 100)).toBe(100)
    expect(percentOf(-5, 100)).toBe(0)
    expect(percentOf(5, 0)).toBe(0)
  })
})

describe('initialsOf', () => {
  test('takes the first letters of the first words', () => {
    expect(initialsOf('Forest Wolf')).toBe('FW')
    expect(initialsOf('Lobo da Floresta', 3)).toBe('LDF')
    expect(initialsOf('  slime ')).toBe('S')
  })
})
