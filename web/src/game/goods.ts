import type { ShopGood } from '../protocol/generated/ShopGood.ts'

/** Fluent message id of the name of an item or a piece of equipment. */
export function goodName(good: ShopGood): string {
  return good.type === 'item' ? `item-${good.id}` : `equipment-${good.id}`
}
