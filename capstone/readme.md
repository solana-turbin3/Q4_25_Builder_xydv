# Recur - Capstone Project

**Recur**  is an on-chain, automated subscription payment solution using blockchain. It allows businesses to create and manage recurring payment plans (e.g., monthly subscriptions) entirely within the decentralized ecosystem. Crucially, it will create a subscriber vault, enabling the service's Program Derived Address (PDA) to automatically deduct tokens from a vault on a set schedule without continuous user interaction or active signatures.

A Merchant defines a plan, the Subscriber subscribes to a subscription and initializes a Vault, and an off-chain automation layer triggers the charge according to the immutable terms of the on-chain contract. This architecture provides reliable, predictable income for Web3 businesses while offering a superior, "set-it-and-forget-it" experience for the user.

### How It Works Simply

1.  **Merchant Setup:** A business sets up a fixed plan (price, token, monthly date).
2.  **Customer Deposits Funds (The Vault):** The customer signs one time to **deposit a prepaid amount of money** into a dedicated **Vault Account** controlled by the system. This Vault acts as the customer's prepaid balance for the subscription.
3.  **Automatic Payment:** When the payment date arrives, a special automated robot triggers the system. The system simply **transfers the necessary funds** directly from the customer's secure Vault to the Merchant.
4.  **No More Signing:** The customer never has to touch their wallet again until they need to add more funds to their Vault or cancel.

---

## Accomplishments (What Works Now on the Test Network)

1.  **Full Payment Cycle Works:** We proved that a business can set up a plan, a customer can **deposit funds into their Vault**, and the system can successfully take the payment and send it to the business.
2.  **Vault Transfers Work:** The core "magic" works: the program (smart contract) can securely access the customer's **Vault** and deduct the payment directly, guaranteeing the funds are available.
3.  **Keeps Track of Time:** The system correctly checks the due date and automatically moves the next payment date forward after a successful charge.
4.  **Fee Separation:** The money is split correctly: the business gets their revenue, and a small service fee goes to the protocol's fee wallet.
