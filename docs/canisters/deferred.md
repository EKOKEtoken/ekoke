# Deferred Canister

- [Deferred Canister](#deferred-canister)
  - [Introduction](#introduction)
  - [Data](#data)
    - [Contract](#contract)
  - [Roles](#roles)
  - [API](#api)
    - [register\_contract](#register_contract)
    - [sign\_contract](#sign_contract)
    - [get\_contract](#get_contract)
    - [get\_token](#get_token)
    - [get\_signed\_contracts](#get_signed_contracts)
    - [get\_agencies](#get_agencies)
    - [remove\_agency](#remove_agency)
    - [get\_unsigned\_contracts](#get_unsigned_contracts)
    - [increment\_contract\_value](#increment_contract_value)
    - [update\_contract\_buyers](#update_contract_buyers)
    - [update\_contract\_property](#update_contract_property)
    - [update\_restricted\_contract\_property](#update_restricted_contract_property)
    - [get\_restricted\_contract\_properties](#get_restricted_contract_properties)
    - [withdraw\_contract\_deposit](#withdraw_contract_deposit)
    - [close\_contract](#close_contract)
    - [admin\_set\_ekoke\_reward\_pool\_canister](#admin_set_ekoke_reward_pool_canister)
    - [admin\_set\_marketplace\_canister](#admin_set_marketplace_canister)
    - [admin\_set\_role](#admin_set_role)
    - [admin\_remove\_role](#admin_remove_role)
    - [admin\_register\_agency](#admin_register_agency)
  - [HTTP API](#http-api)
    - [Request protocol](#request-protocol)
    - [Request Body](#request-body)
    - [HTTP Methods](#http-methods)
      - [getContracts](#getcontracts)
      - [getContract](#getcontract)
      - [getToken](#gettoken)
      - [getAgencies](#getagencies)
  - [Token Metadata](#token-metadata)

![Deferred logo](../../assets/images/deferred-logo.png)

## Introduction

Deferred is a canister which provides a **Non-fungible Token (NFT)** which implements the **DIP-721** Standard <https://github.com/Psychedelic/DIP721/blob/develop/spec.md> with the `dip721_` namespace.

The Deferred canister takes care of registering the sell or financing of a real estate between two or more parts, **Buyers** and **Sellers**. This agreement between parts is called **Contract**.

The Buyer is represented by its Principal, while the Seller is represented by its Principal and its **Quota** in the contract ownership in a total sum of 100.

Each **Contract** is identified by an **ID** (NAT).

Each **Contract** will have Token associated, identified by an incremental **TokenIdentifier** (NAT) as specified by the DIP721 standard.

## Data

### Contract

A Contract is identified by the following properties

- **id**: the contract unique identifier
- **value**: the FIAT value of the contract
- **currency**: the currency used to represent the value
- **agency**: the agency which has created the contract
- **sellers**: the contract sellers. Cannot be empty
- **buyers**: the contract buyers. Cannot be empty. It also contains the deposit account
- **deposit**: buyer deposit amount (FIAT and ICP)
- **is_signed**: if signed the contract tokens can be sold. The token must be signed by custodians (or DAO)
- **type**: the contract type (Sell / Funding)
- **reward**: the reward of EKOKE token given to a NFT buyer
- **expiration**: contract expiration with syntax `YYYY-MM-DD`.
- **properties**: contract properties and metadata
- **restricted_properties**: contract restricted properties

## Roles

On the deferred canister the following roles exists:

- **Custodian**: administrator of the canister, following the DIP721 standard. It can administrate the canister and sign contracts.
- **Agent**: role for agencies. Agents can create contracts, and operate only on contracts created by their agency.

## API

See [DID file](../../src/deferred/deferred.did)

### register_contract

Register a contract with the provided data.

**The contract value** MUST be **multiple of installments**.

### sign_contract

Approve and sign an existing contract. Once signed, the contract's tokens can be sold on the marketplace. Only an admin or an agent can sign the contract.

### get_contract

Get contract by ID

### get_token

Get token and its related contract by ID

### get_signed_contracts

Get the IDS for all the signed contract

### get_agencies

Get all the agencies

### remove_agency

Remove an agency. Only admin or the agent himself can call this method

### get_unsigned_contracts

Get unsigned contracts.
If called by an agent it returns the unsigned contracts for their agency.
If called by an admin it returns all of them.

### increment_contract_value

The seller increments the contract value and mint new NFTs for it

### update_contract_buyers

Update the principal of the contract.

Only the sellers or the buyers can call this method.

### update_contract_property

Change a contract property.

Can be called by Agent, Custodian or seller.

### update_restricted_contract_property

Create or change a contract restricted property.

Can be called by Agent, Custodian or seller.

### get_restricted_contract_properties

Get the restricted contract properties.

The properties returned are those only accessible to the caller

### withdraw_contract_deposit

Update endpoint called by the seller once all the NFTs have been bought by the contract buyers.

The seller provides the contract id he wants to withdraw for and an optional ICRC subaccount.

If the all the NFTs have been paid by the buyer, the deferred canister will transfer the deposit amount to the caller.

The seller will receive only `deposit.value_icp / seller.quota` its part of the deposit.

Each seller must call this method to withdraw their quota of the contract

### close_contract

Update endpoint for the agency to close an expired contract.

All the third party investors are refunded by the liquidity pool with this call, by using the deposit balance. If the deposit balance is not enough, funds from the liquidity pool will be used instead.

### admin_set_ekoke_reward_pool_canister

Update ekoke ledger canister principal

### admin_set_marketplace_canister

Update marketplace canister

### admin_set_role

Set role for principal

### admin_remove_role

Remove a role

### admin_register_agency

Register a new agency in the canister and give to its principal the role of agent

## HTTP API

The deferred canister also exposes an HTTP API

### Request protocol

| Method       | GET              |
|--------------|------------------|
| Content-type | application/json |

### Request Body

The request body must be JSON encoded, and must follow this syntax

| Name   | Type   | Description                                  |
|--------|--------|----------------------------------------------|
| method | string | HTTP method name                             |
| params | json   | key-value map of optional request parameters |

### HTTP Methods

This list contains the http available methods with its parameters and response.

#### getContracts

Get all contracts ids

Body:

```json
{}
```

Response

```json
[
  1,
  2,
]
```

#### getContract

Get a contract by ID

Body:

```json
{
  "id": 1
}
```

Response:

See Contract in did. Returns 404 if it doesn't exist.

#### getToken

Get a token by ID

Body:

```json
{
  "id": 1
}
```

Response:

See TokenInfo in did. Returns 404 if it doesn't exist.

#### getAgencies

Returns all the agencies which have been registered.

## Token Metadata

Each NFT has the following properties, following the DIP721 standard.

- `token:contract_id`: contract id
- `token:value`: fiat value of the contract
- `token:currency`: currency name to represent the value
- `token:ekoke_reward`: reward given when someone buys the contract on the marketplace
- `contract:sellers`: contract sellers principals
- `contract:buyers`: contract buyers principals

> To these properties the contract properties can be added using the syntax `contract:keyname`
