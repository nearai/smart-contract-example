# Smart Contract with Autonomous AI Execution

A NEAR smart contract that implements a fully autonomous AI-powered variant of rock-paper-scissors where players can use any object or concept as their move. The system features a unique architecture where all game data is stored on-chain, with only the AI inference happening off-chain.

## Overview

This contract implements a novel autonomous AI system where:
- All game data, logic, and state is stored entirely on-chain
- An autonomous AI agent triggered by the NEAR AI HUB that monitors the blockchain and automatically processes challenges
- Only the AI inference (LLM model processing) happens off-chain
- Zero manual intervention required - the system runs completely autonomously

## Contact for Autonomous AI Architecture

1. **On-Chain Data Storage**
    - Game state (current champion, history, requests)
    - AI system prompt
    - Agent configuration
    - All player interactions and results
    - Request/response tracking

2. **Autonomous AI Agent**
    - Immutable code stored on NEAR AI HUB
    - HUB Continuously monitors blockchain for corresponding events
    - Automatically processes new challenges without human intervention
    - Uses on-chain data to maintain context and state
    - Only performs LLM model inference off-chain
    - Automatically submits results back to chain
    - Self-contained decision making using stored system prompt

3. **Transaction Flow**
   ```
   [Player Transaction] -> [Smart Contract] -> [Event Emission] ->
   [NEAR AI HUB Detection] -> [Autonomous AI Agent Execution] -> 
   [Fetch On-Chain Data] -> [Off-Chain Inference] ->
   [Automatic Response Transaction] -> [Complete Original Transaction]
   ```

## Key Features

- **Full Autonomy**: System runs 24/7 without any human intervention
- **On-Chain Data**: All agent data and logic stored in smart contract
- **Transparent AI**: All AI decision making parameters stored on-chain
- **Minimal Off-Chain**: Only LLM model inference happens off-chain
- **Automatic Processing**: AI agent automatically handles all challenges

## Smart Contract Storage

The contract maintains all essential data on-chain:

```rust
pub struct Contract {
    // AI Configuration
    agent_name: String,
    agent_public_key: String,
    agent_system_prompt: String,

    // Game State
    current_champion: String,
    champion_owner: AccountId,
    all_champions: UnorderedSet<String>,

    // Request Management
    requests: UnorderedMap<RequestId, Request>,
    responses: LookupMap<RequestId, Response>,
    num_requests: u64,
}
```
## Contract Methods

### State Access Methods
```rust
get_champion()                      // Current champion
get_champion_owner()                // Champion owner address
get_all_champions()                 // Historical champions list
get_question()                      // Current challenge question
get_requests()                      // Pending requests
agent_data(request_id: RequestId)   // Get AI configuration and game data
```

### Game Logic Methods
```rust
request(message: String)  // Submit new challenge
respond(...)              // AI agent response handler
```

## Security and Trust

- All game rules and AI parameters stored on-chain for transparency
- Autonomous agent's public key verified on-chain
- All decisions and their reasoning permanently recorded
- Full audit trail of all games and outcomes

## Example Interaction

```bash
# Player reads current question
near view $CONTRACT_ID get_question
>> What beats steel?

# Player submits challenge
near call $CONTRACT_ID request '{"message": "hummer"}' --accountId player.near

# Autonomous AI agent automatically:
# 1. Detects event
# 2. Fetches current champion "steel", system prompt and list of previous champions
# 3. Processes with LLM completion
# 4. Submits response
# 5. Transaction completes with result:
>> Player player.near won: A Hummer can drive right over steel, and probably over your opponent's ego too.
```
[Transaction with that action](https://nearblocks.io/txns/59xDG5Cnpy1ck1SN9YW32cNcd6vBa68SNaoFJW24erH2#execution)

## DEMO 

- Demo contract: `w00.ai-is-near.near`
- Demo agent: [zavodil.near/what-beats-rock-onchain/latest](https://app.near.ai/agents/zavodil.near/what-beats-rock-onchain/latest/run)