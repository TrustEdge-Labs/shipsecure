/**
 * E2E fixtures for checkout API responses.
 * Kept separate from MSW fixtures used in component tests.
 */

export const checkoutFixtures = {
  success: {
    checkout_url: 'https://checkout.stripe.com/c/pay/cs_test_e2e_mock123',
  },
  error: {
    type: 'about:blank',
    title: 'Failed to create checkout session',
    status: 422,
    detail: 'Invalid scan ID',
  },
} as const;

export type CheckoutFixtures = typeof checkoutFixtures;
