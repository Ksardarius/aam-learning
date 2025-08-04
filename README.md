# **AamLearning: A Decentralized AMM on Solana**

This repository contains a simple, yet robust, Automated Market Maker (AMM) built on the Solana blockchain using the Anchor framework. This project serves as a portfolio piece to demonstrate an understanding of core DeFi concepts, on-chain programming with Rust, and the Solana ecosystem.

## **Project Overview**

The **AamLearning** program implements the fundamental mechanics of a decentralized exchange (DEX) using an AMM model. It allows users to:

* **Initialize a liquidity pool** for any two SPL tokens.  
* **Provide liquidity** to the pool in exchange for LP tokens.  
* **Swap** between the two tokens in the pool.

This project is primarily focused on the on-chain program logic and is fully tested with a comprehensive full-pipeline test suite to ensure correctness and security.

## **Features**

The program exposes the following instructions to interact with the AMM:

* **initialize\_pool**:  
  * Creates a new liquidity pool for a specified pair of SPL tokens (Token A and Token B).  
  * Initializes the associated vaults for storing the tokens.  
  * Sets a custom trading fee percentage for all future swaps.  
* **add\_liquidity**:  
  * Allows a user to deposit an equal value of Token A and Token B into the pool's vaults.  
  * In return, the user receives newly minted LP (Liquidity Provider) tokens, representing their share of the pool.  
  * The amount of LP tokens minted is calculated based on the constant product formula.  
* **swap**:  
  * Enables a user to exchange one token for another.  
  * The price is determined by the ratio of tokens in the pool's vaults.  
  * Includes built-in slippage protection by requiring a minimum amount of output tokens to be specified by the user.

## **Core Concepts**

The AMM uses the popular **constant product market maker** formula:  
x∗y=k  
Where:

* x is the total amount of Token A in the pool.  
* y is the total amount of Token B in the pool.  
* k is a constant value that represents the total liquidity of the pool.

Every swap changes the values of x and y, but the product k must remain the same (minus the trading fee). This ensures that the price of one token relative to the other is always a function of their current ratio in the pool.

## **Getting Started**

This project requires the following tools to be installed:

* [**Rust**](https://www.rust-lang.org/tools/install)  
* [**Solana CLI**](https://www.google.com/search?q=https://docs.solana.com/cli/install-solana-cli)  
* [**Anchor CLI**](https://www.anchor-lang.com/docs/installation)  
* [**Node.js**](https://nodejs.org/) and [**Yarn**](https://yarnpkg.com/)

### **Running the Tests**

To test the full functionality of the program, follow these steps:

1. Clone the repository:  
   git clone https://github.com/your-username/AamLearning.git  
   cd AamLearning

2. Install the JavaScript dependencies for the test suite:  
   yarn install

3. Build the program:  
   anchor build

4. Run the tests. This will automatically start a local validator, deploy the program, and execute the test cases:  
   anchor test

### **Test Case Walkthrough**

The provided test suite, located in tests/aam-learning.ts, demonstrates the complete lifecycle of the AMM. The tests cover:

1. **should create mints**: Verifies the creation of Token A and Token B mints.  
2. **should initialize exchange**: Confirms that a pool is successfully initialized with a trading fee of 30 basis points.  
3. **should create liquidity provider with tokens**: Sets up a new user account and mints a large amount of tokens for them to provide initial liquidity.  
4. **should add liquidity to exchange**: Tests the add\_liquidity instruction, confirming that tokens are deposited and the correct amount of LP tokens are minted and received.  
5. **should user swap token A to token B**: A full swap test case where a user exchanges 10,000 Token A for 8,312 Token B, validating the final token balances.

## **Potential Improvements & Next Steps**

* **Remove Liquidity**: Implement a remove\_liquidity instruction to allow users to burn their LP tokens and withdraw their share of the pool's tokens.  
* **Price Oracle Integration**: Enhance security by integrating a decentralized oracle (e.g., Pyth Network) to validate swap prices against real-world market data and prevent sandwich attacks.  
* **User Interface**: Develop a front-end application to interact with the program, allowing users to see pool statistics, provide liquidity, and perform swaps in a friendly interface.  
* **More Advanced AMM Logic**: Explore other AMM formulas, such as stable swaps for pairs with similar prices, to improve capital efficiency.

## **License**

This project is licensed under the MIT License.

## **Contact**

Feel free to reach out to me with any questions or feedback.
https://www.linkedin.com/in/mihails-orlovs-5b602371
