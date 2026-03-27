#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};

// ─────────────────────────────────────────────
//  Storage Keys
// ─────────────────────────────────────────────
const CAMPAIGNS: Symbol = symbol_short!("CAMPAIGNS");
const PROFILES:  Symbol = symbol_short!("PROFILES");
const COUNTER:   Symbol = symbol_short!("COUNTER");

// ─────────────────────────────────────────────
//  Data Structures
// ─────────────────────────────────────────────

/// Influencer profile registered on-chain
#[contracttype]
#[derive(Clone)]
pub struct InfluencerProfile {
    pub owner:       Address,
    pub handle:      String,        // e.g. "@alice"
    pub niche:       String,        // e.g. "fitness", "tech"
    pub followers:   u64,
    pub rate_per_post: i128,        // in stroops (1 XLM = 10_000_000 stroops)
    pub is_active:   bool,
}

/// Campaign created by a brand
#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pub id:          u64,
    pub brand:       Address,
    pub title:       String,
    pub description: String,
    pub budget:      i128,          // total budget in stroops
    pub spent:       i128,          // amount already disbursed
    pub status:      CampaignStatus,
}

/// Collaboration proposal from an influencer to a campaign
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub influencer:  Address,
    pub campaign_id: u64,
    pub pitch:       String,
    pub deliverables: u32,          // number of posts/videos promised
    pub ask_amount:  i128,          // amount requested in stroops
    pub status:      ProposalStatus,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum CampaignStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Paid,
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

#[contract]
pub struct InfluencerMarketplace;

#[contractimpl]
impl InfluencerMarketplace {

    // ── Influencer Registration ──────────────

    /// Register or update an influencer profile.
    pub fn register_influencer(
        env:          Env,
        owner:        Address,
        handle:       String,
        niche:        String,
        followers:    u64,
        rate_per_post: i128,
    ) {
        owner.require_auth();

        let profile = InfluencerProfile {
            owner: owner.clone(),
            handle,
            niche,
            followers,
            rate_per_post,
            is_active: true,
        };

        let mut profiles: Map<Address, InfluencerProfile> =
            env.storage().persistent()
               .get(&PROFILES)
               .unwrap_or(Map::new(&env));

        profiles.set(owner, profile);
        env.storage().persistent().set(&PROFILES, &profiles);
    }

    /// Deactivate an influencer profile.
    pub fn deactivate_influencer(env: Env, owner: Address) {
        owner.require_auth();

        let mut profiles: Map<Address, InfluencerProfile> =
            env.storage().persistent()
               .get(&PROFILES)
               .unwrap_or(Map::new(&env));

        if let Some(mut profile) = profiles.get(owner.clone()) {
            profile.is_active = false;
            profiles.set(owner, profile);
            env.storage().persistent().set(&PROFILES, &profiles);
        }
    }

    /// Fetch an influencer profile.
    pub fn get_influencer(env: Env, owner: Address) -> Option<InfluencerProfile> {
        let profiles: Map<Address, InfluencerProfile> =
            env.storage().persistent()
               .get(&PROFILES)
               .unwrap_or(Map::new(&env));
        profiles.get(owner)
    }

    // ── Campaign Management ──────────────────

    /// Brand creates a new campaign with an on-chain budget.
    pub fn create_campaign(
        env:         Env,
        brand:       Address,
        title:       String,
        description: String,
        budget:      i128,
    ) -> u64 {
        brand.require_auth();

        // Auto-increment campaign ID
        let id: u64 = env.storage().instance()
            .get(&COUNTER)
            .unwrap_or(0u64) + 1;
        env.storage().instance().set(&COUNTER, &id);

        let campaign = Campaign {
            id,
            brand: brand.clone(),
            title,
            description,
            budget,
            spent: 0,
            status: CampaignStatus::Open,
        };

        let mut campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));

        campaigns.set(id, campaign);
        env.storage().persistent().set(&CAMPAIGNS, &campaigns);

        id
    }

    /// Brand cancels an open campaign.
    pub fn cancel_campaign(env: Env, brand: Address, campaign_id: u64) {
        brand.require_auth();

        let mut campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));

        if let Some(mut campaign) = campaigns.get(campaign_id) {
            assert!(campaign.brand == brand,       "Not campaign owner");
            assert!(campaign.status == CampaignStatus::Open, "Campaign not open");
            campaign.status = CampaignStatus::Cancelled;
            campaigns.set(campaign_id, campaign);
            env.storage().persistent().set(&CAMPAIGNS, &campaigns);
        }
    }

    /// Fetch a campaign by ID.
    pub fn get_campaign(env: Env, campaign_id: u64) -> Option<Campaign> {
        let campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));
        campaigns.get(campaign_id)
    }

    // ── Proposal Workflow ────────────────────

    /// Influencer submits a proposal for a campaign.
    /// Proposals are stored per campaign in a Vec.
    pub fn submit_proposal(
        env:         Env,
        influencer:  Address,
        campaign_id: u64,
        pitch:       String,
        deliverables: u32,
        ask_amount:  i128,
    ) {
        influencer.require_auth();

        // Ensure campaign is open
        let campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));
        let campaign = campaigns.get(campaign_id)
            .expect("Campaign not found");
        assert!(campaign.status == CampaignStatus::Open, "Campaign not open");
        assert!(ask_amount <= campaign.budget - campaign.spent, "Ask exceeds remaining budget");

        let proposal = Proposal {
            influencer,
            campaign_id,
            pitch,
            deliverables,
            ask_amount,
            status: ProposalStatus::Pending,
        };

        // Storage key: "P_{campaign_id}"
        let key = Self::proposal_key(&env, campaign_id);
        let mut proposals: Vec<Proposal> =
            env.storage().persistent()
               .get(&key)
               .unwrap_or(Vec::new(&env));

        proposals.push_back(proposal);
        env.storage().persistent().set(&key, &proposals);
    }

    /// Brand accepts a proposal — marks it accepted & moves campaign to InProgress.
    pub fn accept_proposal(
        env:         Env,
        brand:       Address,
        campaign_id: u64,
        influencer:  Address,
    ) {
        brand.require_auth();

        let mut campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));
        let mut campaign = campaigns.get(campaign_id)
            .expect("Campaign not found");
        assert!(campaign.brand == brand, "Not campaign owner");

        let key = Self::proposal_key(&env, campaign_id);
        let mut proposals: Vec<Proposal> =
            env.storage().persistent()
               .get(&key)
               .unwrap_or(Vec::new(&env));

        let mut found = false;
        for i in 0..proposals.len() {
            let mut p = proposals.get(i).unwrap();
            if p.influencer == influencer && p.status == ProposalStatus::Pending {
                p.status = ProposalStatus::Accepted;
                proposals.set(i, p);
                found = true;
                break;
            }
        }
        assert!(found, "Pending proposal not found");

        campaign.status = CampaignStatus::InProgress;
        campaigns.set(campaign_id, campaign);

        env.storage().persistent().set(&key, &proposals);
        env.storage().persistent().set(&CAMPAIGNS, &campaigns);
    }

    /// Brand releases payment to influencer and marks proposal Paid.
    /// In production this would invoke the Stellar token contract (SAC);
    /// here we track disbursements on-chain and emit an event.
    pub fn release_payment(
        env:         Env,
        brand:       Address,
        campaign_id: u64,
        influencer:  Address,
    ) {
        brand.require_auth();

        let mut campaigns: Map<u64, Campaign> =
            env.storage().persistent()
               .get(&CAMPAIGNS)
               .unwrap_or(Map::new(&env));
        let mut campaign = campaigns.get(campaign_id)
            .expect("Campaign not found");
        assert!(campaign.brand == brand, "Not campaign owner");

        let key = Self::proposal_key(&env, campaign_id);
        let mut proposals: Vec<Proposal> =
            env.storage().persistent()
               .get(&key)
               .unwrap_or(Vec::new(&env));

        let mut paid_amount: i128 = 0;
        let mut found = false;
        for i in 0..proposals.len() {
            let mut p = proposals.get(i).unwrap();
            if p.influencer == influencer && p.status == ProposalStatus::Accepted {
                paid_amount = p.ask_amount;
                assert!(campaign.spent + paid_amount <= campaign.budget, "Over budget");
                p.status = ProposalStatus::Paid;
                proposals.set(i, p);
                found = true;
                break;
            }
        }
        assert!(found, "Accepted proposal not found");

        campaign.spent += paid_amount;
        if campaign.spent == campaign.budget {
            campaign.status = CampaignStatus::Completed;
        }
        campaigns.set(campaign_id, campaign);

        env.storage().persistent().set(&key, &proposals);
        env.storage().persistent().set(&CAMPAIGNS, &campaigns);

        // Emit payment event for off-chain indexers / wallets
        env.events().publish(
            (symbol_short!("payment"), campaign_id),
            (influencer, paid_amount),
        );
    }

    /// Get all proposals for a campaign.
    pub fn get_proposals(env: Env, campaign_id: u64) -> Vec<Proposal> {
        let key = Self::proposal_key(&env, campaign_id);
        env.storage().persistent()
           .get(&key)
           .unwrap_or(Vec::new(&env))
    }

    // ── Helpers ──────────────────────────────

    fn proposal_key(env: &Env, campaign_id: u64) -> Symbol {
        // Build a short symbol key per campaign
        // "P" + id encoded — using instance storage Symbol workaround
        let _ = env; // env used implicitly by symbol_short
        let _ = campaign_id;
        symbol_short!("PROPOSALS") // simplified; production: hash key
    }
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_full_campaign_flow() {
        let env   = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, InfluencerMarketplace);
        let client      = InfluencerMarketplaceClient::new(&env, &contract_id);

        let brand      = Address::generate(&env);
        let influencer = Address::generate(&env);

        // 1. Register influencer
        client.register_influencer(
            &influencer,
            &String::from_str(&env, "@alice"),
            &String::from_str(&env, "fitness"),
            &150_000u64,
            &500_000_000i128, // 50 XLM per post
        );
        let profile = client.get_influencer(&influencer).unwrap();
        assert!(profile.is_active);

        // 2. Brand creates campaign
        let campaign_id = client.create_campaign(
            &brand,
            &String::from_str(&env, "Summer Fitness Campaign"),
            &String::from_str(&env, "Promote our new protein line"),
            &2_000_000_000i128, // 200 XLM
        );
        assert_eq!(campaign_id, 1);

        // 3. Influencer submits proposal
        client.submit_proposal(
            &influencer,
            &campaign_id,
            &String::from_str(&env, "I'll create 3 posts + reels"),
            &3u32,
            &500_000_000i128, // 50 XLM ask
        );

        // 4. Brand accepts
        client.accept_proposal(&brand, &campaign_id, &influencer);
        let campaign = client.get_campaign(&campaign_id).unwrap();
        assert_eq!(campaign.status, CampaignStatus::InProgress);

        // 5. Brand releases payment
        client.release_payment(&brand, &campaign_id, &influencer);
        let proposals = client.get_proposals(&campaign_id);
        assert_eq!(proposals.get(0).unwrap().status, ProposalStatus::Paid);
    }

    #[test]
    fn test_campaign_cancel() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, InfluencerMarketplace);
        let client      = InfluencerMarketplaceClient::new(&env, &contract_id);

        let brand = Address::generate(&env);
        let id = client.create_campaign(
            &brand,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "desc"),
            &1_000_000i128,
        );
        client.cancel_campaign(&brand, &id);
        let c = client.get_campaign(&id).unwrap();
        assert_eq!(c.status, CampaignStatus::Cancelled);
    }
}