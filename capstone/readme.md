# Recur - Automated Subscriptions Service

**Recur**  is an on-chain, automated subscription payment solution using blockchain. It allows businesses to create and manage recurring payment plans (e.g., monthly subscriptions) entirely within the decentralized ecosystem. Crucially, it will create a subscriber vault, enabling the service's Program Derived Address (PDA) to automatically deduct tokens from a vault on a set schedule without continuous user interaction or active signatures.

### How It Works Simply

1.  **Merchant Setup:** A business sets up a fixed plan (price, charge interval, token, etc).
2.  **Customer Subscribes:** The customer signs one time to subscribe to the plan and creates a vault.
3. **Tuktuk Tasks**: Creates automatic tasks for recurring payments
4.  **Automatic Payment:** When the payment date arrives, Tuktuk triggers the charge automatically.
5.  **No More Signing:**: The customer never has to touch their wallet again.
---

## Accomplishments (What Works Now on the Test Network)

1.  **Full Payment Cycle Works:** The business can set up a plan, a customer can **deposit funds into their Vault**, and the system can successfully take the payment and send it to the business .
2.  **Vault Transfers Work:** The program (smart contract) can securely access the customer's **Vault** and deduct the payment directly, guaranteeing the funds are available.
3. **Automation Works**: Tuktuk charges the subscriber according to the plan and the interval mentioned.
4.  **Keeps Track of Time:** The system correctly checks the due date and automatically moves the next payment date forward after a successful charge.
5.  **Fee Separation:** The money is split correctly: the business gets their revenue, and a small service fee goes to the protocol's fee wallet.

## Devnet Deployment

Program ID: `ZtzPHWinzmfmxDBeoEUy2JDSt3qGp3pv1BuAFc6nrop`

[Solana Explorer](https://explorer.solana.com/address/ZtzPHWinzmfmxDBeoEUy2JDSt3qGp3pv1BuAFc6nrop?cluster=devnet)

## Features

### Core Functionality

* **Create Subscription** - Merchant can create a subscription
* **User Subscribes** - User can subscribe to a plan
* **Tuktuk Automation** - Tuktuk automatically charges user based on plan interval
* **User Specific Vault** - Vault for each user to store USDC tokens with full authority
* **Automated Cancellations** - Subscriptions cancels when max failure counts are reached

### Future Features

* **User Interface** - A user interface to manage all subscriptions
* **NFT's for Subscriptions** - A NFT for each subscription plan (for Subscriber)
* **Payment Notifications** - Real-time payment alerts on Telegram

## Architecture

### State Accounts

* **GlobalState**: Global configuration storing task queue, task queue authority and fees
* **SubscriptionPlan**: A subscription plan schema, stores plan related data.
* **UserSubscription**: A state for each user subscription, stores plan data along with next timestamp, etc.

### Core Instructions

1. **Initialize** - Set up global configuration for protocol
2. **Create Subscription** - Merchant creates a new subscription plan
3. **Subscribe** - Customer subscribes to a plan
4. **Charge User** - Tuktuk calls this instruction to recursively create tasks
5. **Cancel Subscription** - Cancel the user subscription and close the PDA
6. **Close Vault** - Close the vault token account

## Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
anchor test

# Run devnet tests
anchor run devnet

```

## Demos

The project includes demo tests:

```bash
# Run create subscription demo
anchor run create-subscription

# Run subscribe demo
anchor run subscribe

# Run cancel subscription demo
anchor run cancel-subscription

# Run close vault demo
anchor run close-vault
```

### Test Coverage

* Global config initialization
* Create subscription
* User subscribes to a plan
* Cancel subscription
* Close vault
* Authorization checks
* Insufficient USDC tokens handling
* Boundary cases and error conditions

## Assignments

All Turbin3 Assignments and Architechure diagrams are in `documents/` folder.

## Authors

* **xydv** - Core Developer - [@xydv](https://github.com/xydv)
