import { useCallback, useEffect, useState } from "react";
import { api, Proposal, VoteOption, StatusResponse } from "./api";
import { connectWallet, disconnectWallet, getActiveAddress, getFreighterNetwork } from "./wallet";
import { getXlmBalance, NETWORK_NAME } from "./stellar";
import "./styles.css";

type Tab = "dashboard" | "proposals" | "create" | "vote" | "admin" | "wallet";
type Toast = { kind: "success" | "error" | "info"; msg: string } | null;

export default function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [dark, setDark] = useState(() => localStorage.getItem("dark") === "1");
  const [toast, setToast] = useState<Toast>(null);
  const [addr, setAddr] = useState<string | null>(null);
  const [net, setNet] = useState<string | null>(null);
  const [bal, setBal] = useState<string | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);

  useEffect(() => { document.documentElement.className = dark ? "dark" : ""; localStorage.setItem("dark", dark ? "1" : "0"); }, [dark]);
  useEffect(() => { if (toast) { const t = setTimeout(() => setToast(null), 4000); return () => clearTimeout(t); } }, [toast]);
  useEffect(() => { (async () => { const a = await getActiveAddress(); if (a) { setAddr(a); const n = await getFreighterNetwork(); setNet(n?.network ?? null); } })(); }, []);

  const refresh = useCallback(async () => { try { setStatus(await api.status()); } catch {} }, []);
  useEffect(() => { refresh(); const i = setInterval(refresh, 30000); return () => clearInterval(i); }, [refresh]);
  const showToast = (k: "success" | "error" | "info", m: string) => setToast({ kind: k, msg: m });

  const onConnect = async () => {
    const r = await connectWallet();
    if (r.error) return showToast("error", r.error);
    setAddr(r.address!); const n = await getFreighterNetwork(); setNet(n?.network ?? null);
    try { setBal(await getXlmBalance(r.address!)); } catch {}
    showToast("success", "Wallet connected");
  };
  const onDisconnect = async () => { await disconnectWallet(); setAddr(null); setNet(null); setBal(null); showToast("info", "Disconnected"); };

  return (
    <div className="app">
      <header className="header">
        <div className="header-left"><h1 className="logo">🗳️ Governance</h1><span className="badge">v{status?.version || "—"}</span></div>
        <div className="header-right">
          <button className="icon-btn" onClick={() => setDark(!dark)}>{dark ? "☀️" : "🌙"}</button>
          {addr ? <div className="wallet-pill"><code>{addr.slice(0, 6)}…{addr.slice(-4)}</code><button onClick={onDisconnect}>Disconnect</button></div> : <button className="primary" onClick={onConnect}>Connect Freighter</button>}
        </div>
      </header>
      <nav className="tabs">
        {(["dashboard", "proposals", "create", "vote", "admin", "wallet"] as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "tab active" : "tab"} onClick={() => setTab(t)}>
            {t === "dashboard" && "📊 Dashboard"}{t === "proposals" && "📋 Proposals"}{t === "create" && "✏️ Create"}{t === "vote" && "🗳️ Vote"}{t === "admin" && "⚙️ Admin"}{t === "wallet" && "👛 Wallet"}
          </button>
        ))}
      </nav>
      <main className="main">
        {tab === "dashboard" && <Dashboard status={status} onRefresh={refresh} />}
        {tab === "proposals" && <ProposalsView showToast={showToast} />}
        {tab === "create" && <CreateForm proposer={addr} showToast={showToast} onCreated={refresh} />}
        {tab === "vote" && <VoteView addr={addr} showToast={showToast} />}
        {tab === "admin" && <AdminPanel showToast={showToast} onRefresh={refresh} />}
        {tab === "wallet" && <WalletView addr={addr} net={net} bal={bal} onConnect={onConnect} onRefresh={async () => { if (addr) try { setBal(await getXlmBalance(addr)); } catch {} }} />}
      </main>
      {toast && <div className={`toast ${toast.kind}`}>{toast.kind === "success" ? "✅" : toast.kind === "error" ? "❌" : "ℹ️"} {toast.msg}</div>}
      <footer className="footer"><p>Stellar Governance Voting · Testnet · MIT License</p></footer>
    </div>
  );
}

function Dashboard({ status, onRefresh }: { status: StatusResponse | null; onRefresh: () => void }) {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  useEffect(() => { api.listProposals().then(p => setProposals(p.slice(0, 5))).catch(() => {}); }, [status?.proposal_count]);
  return (
    <div>
      <div className="stats-grid">
        <div className="stat-card"><span className="stat-label">Proposals</span><span className="stat-value">{status?.proposal_count ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Voters</span><span className="stat-value">{status?.voter_count ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Network</span><span className="stat-value">{status?.network ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Uptime</span><span className="stat-value">{status ? `${Math.floor(status.uptime_seconds / 60)}m` : "—"}</span></div>
      </div>
      <div className="card"><h2>Recent Proposals</h2>
        {proposals.length === 0 ? <p className="muted">No proposals yet.</p> : (
          <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Title</th><th>Yes</th><th>No</th><th>Abstain</th><th>Status</th></tr></thead>
          <tbody>{proposals.map((p) => (<tr key={p.id}><td>{p.id}</td><td className="memo-cell">{p.title}</td><td>{p.yes_votes}</td><td>{p.no_votes}</td><td>{p.abstain_votes}</td><td><span className={`status status-${p.status}`}>{p.status}</span></td></tr>))}</tbody></table></div>
        )}
      </div>
      <button onClick={onRefresh}>🔄 Refresh</button>
    </div>
  );
}

function ProposalsView({ showToast }: { showToast: (k: "success" | "error" | "info", m: string) => void }) {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<Proposal | null>(null);
  const load = useCallback(async () => { setLoading(true); try { setProposals(await api.listProposals()); } catch (e: any) { showToast("error", e.message); } finally { setLoading(false); } }, [showToast]);
  useEffect(() => { load(); }, [load]);
  return (
    <div className="card"><h2>All Proposals</h2>
      {loading ? <p className="muted">Loading…</p> : proposals.length === 0 ? <p className="muted">No proposals.</p> : (
        <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Title</th><th>Proposer</th><th>Start</th><th>End</th><th>Yes</th><th>No</th><th>Abstain</th><th>Status</th></tr></thead>
        <tbody>{proposals.map((p) => (<tr key={p.id} onClick={() => setSelected(p)} className="clickable">
          <td>{p.id}</td><td className="memo-cell">{p.title}</td><td><code className="addr">{p.proposer.slice(0,10)}…</code></td>
          <td className="ts">{new Date(p.start_time * 1000).toLocaleDateString()}</td><td className="ts">{new Date(p.end_time * 1000).toLocaleDateString()}</td>
          <td>{p.yes_votes}</td><td>{p.no_votes}</td><td>{p.abstain_votes}</td>
          <td><span className={`status status-${p.status}`}>{p.status}</span></td>
        </tr>))}</tbody></table></div>
      )}
      {selected && (
        <div className="modal-overlay" onClick={() => setSelected(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>Proposal #{selected.id}</h2>
            <h3>{selected.title}</h3>
            <p className="muted">{selected.description}</p>
            <div className="detail-grid">
              <div className="detail-row"><span className="detail-label">Proposer:</span><code className="mono">{selected.proposer}</code></div>
              <div className="detail-row"><span className="detail-label">Start:</span>{new Date(selected.start_time * 1000).toISOString()}</div>
              <div className="detail-row"><span className="detail-label">End:</span>{new Date(selected.end_time * 1000).toISOString()}</div>
              <div className="detail-row"><span className="detail-label">Yes:</span><strong>{selected.yes_votes}</strong></div>
              <div className="detail-row"><span className="detail-label">No:</span><strong>{selected.no_votes}</strong></div>
              <div className="detail-row"><span className="detail-label">Abstain:</span><strong>{selected.abstain_votes}</strong></div>
              <div className="detail-row"><span className="detail-label">Status:</span><span className={`status status-${selected.status}`}>{selected.status}</span></div>
            </div>
            <button onClick={() => setSelected(null)}>Close</button>
          </div>
        </div>
      )}
    </div>
  );
}

function CreateForm({ proposer, showToast, onCreated }: { proposer: string | null; showToast: (k: "success" | "error" | "info", m: string) => void; onCreated: () => void }) {
  const [title, setTitle] = useState("");
  const [desc, setDesc] = useState("");
  const [duration, setDuration] = useState("3600");
  const [submitting, setSubmitting] = useState(false);
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!proposer) return showToast("error", "Connect wallet first");
    if (!title) return showToast("error", "Title required");
    setSubmitting(true);
    try { const r = await api.createProposal(title, desc, proposer, parseInt(duration)); showToast("success", r.message); setTitle(""); setDesc(""); onCreated(); }
    catch (e: any) { showToast("error", e.message); } finally { setSubmitting(false); }
  };
  return (
    <div className="card"><h2>Create Proposal</h2>
      <p className="muted">Proposer: {proposer ? <code>{proposer.slice(0,12)}…</code> : "⚠️ Connect wallet"}</p>
      <form onSubmit={handleSubmit} className="form">
        <label>Title<input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Proposal title" maxLength={128} /></label>
        <label>Description<textarea value={desc} onChange={(e) => setDesc(e.target.value)} placeholder="Detailed proposal description…" rows={4} maxLength={1024} /></label>
        <label>Voting Duration (seconds)<input type="number" value={duration} onChange={(e) => setDuration(e.target.value)} min="60" /></label>
        <button type="submit" className="primary" disabled={submitting || !proposer}>{submitting ? "Creating…" : "Create Proposal"}</button>
      </form>
    </div>
  );
}

function VoteView({ addr, showToast }: { addr: string | null; showToast: (k: "success" | "error" | "info", m: string) => void }) {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [voting, setVoting] = useState<number | null>(null);
  const load = useCallback(async () => { try { setProposals(await api.listProposals()); } catch {} }, []);
  useEffect(() => { load(); }, [load]);
  const activeProposals = proposals.filter(p => p.status === "active");
  const handleVote = async (pid: number, option: VoteOption) => {
    if (!addr) return showToast("error", "Connect wallet first");
    setVoting(pid);
    try { await api.vote(addr, pid, option); showToast("success", "Vote recorded!"); load(); }
    catch (e: any) { showToast("error", e.message); } finally { setVoting(null); }
  };
  return (
    <div>
      <div className="card"><h2>Active Proposals — Cast Your Vote</h2>
        {activeProposals.length === 0 ? <p className="muted">No active proposals to vote on.</p> : activeProposals.map((p) => (
          <div key={p.id} className="proposal-vote-card">
            <h3>#{p.id}: {p.title}</h3>
            <p className="muted">{p.description}</p>
            <div className="vote-results">
              <span className="vote-bar">✅ Yes: {p.yes_votes}</span>
              <span className="vote-bar">❌ No: {p.no_votes}</span>
              <span className="vote-bar">⚪ Abstain: {p.abstain_votes}</span>
            </div>
            <p className="muted">Ends: {new Date(p.end_time * 1000).toLocaleString()}</p>
            <div className="actions-row">
              <button className="primary" disabled={voting === p.id || !addr} onClick={() => handleVote(p.id, "yes")}>Vote Yes</button>
              <button disabled={voting === p.id || !addr} onClick={() => handleVote(p.id, "no")}>Vote No</button>
              <button disabled={voting === p.id || !addr} onClick={() => handleVote(p.id, "abstain")}>Abstain</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function AdminPanel({ showToast, onRefresh }: { showToast: (k: "success" | "error" | "info", m: string) => void; onRefresh: () => void }) {
  const [paused, setPaused] = useState(false);
  const [checkAddr, setCheckAddr] = useState("");
  const [voterResult, setVoterResult] = useState<boolean | null>(null);
  const [registerAddr, setRegisterAddr] = useState("");
  const [proposals, setProposals] = useState<Proposal[]>([]);
  useEffect(() => { api.isPaused().then(r => setPaused(r.paused)).catch(() => {}); api.listProposals().then(setProposals).catch(() => {}); }, []);
  const togglePause = async () => { try { if (paused) { await api.unpause(); setPaused(false); showToast("success", "Resumed"); } else { await api.pause(); setPaused(true); showToast("info", "Paused"); } onRefresh(); } catch (e: any) { showToast("error", e.message); } };
  const checkV = async () => { try { const r = await api.checkVoter(checkAddr); setVoterResult(r.is_eligible); } catch (e: any) { showToast("error", e.message); } };
  const registerV = async () => { try { await api.registerVoter(registerAddr); showToast("success", "Voter registered"); setRegisterAddr(""); } catch (e: any) { showToast("error", e.message); } };
  const closeP = async (id: number) => { try { await api.closeProposal(id); showToast("success", `Proposal ${id} closed`); api.listProposals().then(setProposals); } catch (e: any) { showToast("error", e.message); } };
  const cancelP = async (id: number) => { try { await api.cancelProposal(id); showToast("info", `Proposal ${id} cancelled`); api.listProposals().then(setProposals); } catch (e: any) { showToast("error", e.message); } };
  return (
    <div className="admin-panel">
      <div className="card"><h2>Protocol Controls</h2>
        <div className="detail-row"><span className="detail-label">Status:</span><span className={paused ? "warn" : "ok"}>{paused ? "⏸ Paused" : "✅ Active"}</span></div>
        <button className={paused ? "primary" : ""} onClick={togglePause}>{paused ? "▶️ Resume" : "⏸ Pause"}</button>
      </div>
      <div className="card"><h2>Voter Management</h2>
        <div className="inline-form"><input value={checkAddr} onChange={(e) => setCheckAddr(e.target.value)} placeholder="G… address to check" /><button onClick={checkV}>Check</button></div>
        {voterResult !== null && <p className={voterResult ? "ok" : "warn"}>{voterResult ? "✅ Eligible voter" : "❌ Not registered"}</p>}
        <div className="inline-form"><input value={registerAddr} onChange={(e) => setRegisterAddr(e.target.value)} placeholder="G… to register" /><button className="primary" onClick={registerV}>Register</button></div>
      </div>
      <div className="card"><h2>Manage Proposals</h2>
        {proposals.map((p) => (
          <div key={p.id} className="claim-item">
            <p><strong>#{p.id}: {p.title}</strong> · <span className={`status status-${p.status}`}>{p.status}</span></p>
            {p.status === "active" && <div className="actions-row"><button className="primary" onClick={() => closeP(p.id)}>Close</button><button onClick={() => cancelP(p.id)}>Cancel</button></div>}
          </div>
        ))}
      </div>
    </div>
  );
}

function WalletView({ addr, net, bal, onConnect, onRefresh }: { addr: string | null; net: string | null; bal: string | null; onConnect: () => void; onRefresh: () => void }) {
  return (
    <div className="card"><h2>Freighter Wallet</h2>
      {!addr ? (<div><p className="muted">Connect to vote and create proposals.</p><button className="primary" onClick={onConnect}>Connect Freighter</button></div>) : (
        <div>
          <div className="detail-row"><span className="detail-label">Address:</span><code className="mono">{addr}</code></div>
          <div className="detail-row"><span className="detail-label">Network:</span><span className={net === NETWORK_NAME ? "ok" : "warn"}>{net ?? "unknown"}{net !== NETWORK_NAME ? " (switch to Testnet!)" : ""}</span></div>
          <div className="detail-row"><span className="detail-label">XLM:</span><strong>{bal ?? "—"}</strong></div>
          <button onClick={onRefresh}>🔄 Refresh Balance</button>
        </div>
      )}
    </div>
  );
}
