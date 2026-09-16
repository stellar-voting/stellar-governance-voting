const BASE = "/api/v1";

export type ProposalStatus = "active" | "closed" | "cancelled";
export type VoteOption = "yes" | "no" | "abstain";

export interface Proposal {
  id: number; title: string; description: string; proposer: string;
  start_time: number; end_time: number; status: ProposalStatus;
  yes_votes: number; no_votes: number; abstain_votes: number; ledger: number;
}

export interface StatusResponse {
  status: string; version: string; uptime_seconds: number;
  network: string; contract_id: string;
  proposal_count: number; voter_count: number;
}

async function jsonFetch<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, { headers: { "Content-Type": "application/json" }, ...init });
  if (!res.ok) { const e = await res.json().catch(() => ({ message: res.statusText })); throw new Error(e.message || `HTTP ${res.status}`); }
  return res.json();
}

export const api = {
  health: () => jsonFetch<{ status: string; version: string }>("​/health"),
  status: () => jsonFetch<StatusResponse>(`${BASE}/status`),
  listProposals: () => jsonFetch<Proposal[]>(`${BASE}/proposals`),
  getProposal: (id: number) => jsonFetch<Proposal>(`${BASE}/proposals/${id}`),
  proposalCount: () => jsonFetch<{ count: number }>(`${BASE}/proposals/count`),
  createProposal: (title: string, description: string, proposer: string, voting_duration: number) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/proposals`, { method: "POST", body: JSON.stringify({ title, description, proposer, voting_duration }) }),
  vote: (voter: string, proposal_id: number, option: VoteOption) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/vote`, { method: "POST", body: JSON.stringify({ voter, proposal_id, option }) }),
  getResults: (id: number) => jsonFetch<{ yes: number; no: number; abstain: number }>(`${BASE}/proposals/${id}/results`),
  closeProposal: (id: number) => jsonFetch(`${BASE}/proposals/${id}/close`, { method: "POST" }),
  cancelProposal: (id: number) => jsonFetch(`${BASE}/proposals/${id}/cancel`, { method: "POST" }),
  voterCount: () => jsonFetch<{ count: number }>(`${BASE}/voters/count`),
  registerVoter: (voter: string) => jsonFetch(`${BASE}/voters/register`, { method: "POST", body: JSON.stringify({ voter }) }),
  checkVoter: (addr: string) => jsonFetch<{ address: string; is_eligible: boolean }>(`${BASE}/voters/${addr}`),
  hasVoted: (addr: string, pid: number) => jsonFetch<{ has_voted: boolean }>(`${BASE}/voted/${addr}/${pid}`),
  getAdmin: () => jsonFetch<{ admin: string }>(`${BASE}/admin`),
  pause: () => jsonFetch(`${BASE}/pause`, { method: "POST" }),
  unpause: () => jsonFetch(`${BASE}/unpause`, { method: "POST" }),
  isPaused: () => jsonFetch<{ paused: boolean }>(`${BASE}/paused`),
};
