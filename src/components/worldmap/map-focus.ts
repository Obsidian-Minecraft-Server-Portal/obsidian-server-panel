/** How a followed player is framed: from behind them, or from inside their head. */
export type FollowMode = "behind" | "head";

/**
 * The player the views centre on. A new `nonce` moves the views to them once. `follow` keeps them on the player as they
 * move, and `null` leaves the view free.
 */
export interface MapFocus {
  name: string;
  nonce: number;
  follow: FollowMode | null;
}
