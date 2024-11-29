# Solana Escrow Game: A Price Prediction Challenge

The Solana Escrow Game is an interactive platform for two players to compete in a fun and strategic game based on price movements of ETH against USDC. Players predict whether the ETH price will increase or decrease by 5% first, and the winner claims the prize. The game relies on an escrow contract to securely manage the players' funds and ensure fairness.

## System Overview

The game involves two players who contribute an entry fee to participate. The core idea is simple: players make opposing predictions about ETH price movement, and the first movement of 5% in their chosen direction determines the winner. The system tracks price changes and handles funds securely, ensuring transparency and fairness.

### Game Flow

#### **1. Player 1 Enters the Game**

- The first player starts the game by selecting a direction: either **increase by 5%** or **decrease by 5%**.
- They pay an entry fee, which is held securely in the escrow contract.

#### **2. Player 2 Joins the Game**

- A second player can join only if they select the opposite direction to Player 1.
- Their entry fee is also added to the escrow contract.
- A crucial condition: Player 2 can only join if the ETH price hasn’t fluctuated by more than 1% since Player 1 entered the game. This ensures fairness and a relatively stable starting point for predictions.

#### **3. Withdrawal Option for Player 1**

- If no second player joins, Player 1 can withdraw their entry fee without penalty.

#### **4. Lock-in Phase**

- Once Player 2 joins, both players' funds are locked, and neither can withdraw.

#### **5. Closing the Game**

- As the ETH price fluctuates, the escrow contract monitors it.
- If a 5% increase or decrease occurs, the winning player can call the **closeGame** function.
- The escrow contract verifies the price movement and sends the pooled funds to the winner.

The Solana Escrow Game is a combination of strategic thinking and price movement analysis, making it an engaging challenge for players while leveraging the reliability of blockchain technology to ensure trust and transparency.
