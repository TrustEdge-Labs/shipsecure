export const checkoutFixtures = {
  success: {
    checkout_url: 'https://checkout.stripe.com/c/pay/cs_test_abc123def456',
  },

  error: {
    type: 'https://shipsecure.io/errors/checkout-failed',
    title: 'Checkout Failed',
    status: 500,
    detail: 'Unable to create checkout session. Please try again.',
  },
} as const
