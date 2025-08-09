"use client";
import { useEffect, useRef, useState } from "react";

export default function LookupPage() {
  const [handle, setHandle] = useState("");
  const [resp, setResp] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const pollRef = useRef<any>(null);
  const API_BASE = process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8080";

  async function submit() {
    setLoading(true);
    setResp(null);
    const r = await fetch(`${API_BASE}/v1/lookup`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ handle, force: false }),
    });
    const j = await r.json();
    setResp(j);
    setLoading(false);
    // Begin polling job status until done
    if (j?.jobId) {
      if (pollRef.current) clearInterval(pollRef.current);
      pollRef.current = setInterval(async () => {
        try {
          const s = await fetch(`${API_BASE}/internal/jobs/score/${j.jobId}`);
          const sj = await s.json();
          setResp((prev: any) => ({ ...prev, status: sj.status || prev?.status }));
          if (sj.status === "done") {
            clearInterval(pollRef.current);
          }
        } catch {}
      }, 800);
    }
  }

  useEffect(() => {
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-semibold mb-4">Lookup & Score</h1>
      <input
        value={handle}
        onChange={(e) => setHandle(e.target.value)}
        placeholder="nharan.bsky.social"
        className="border rounded px-3 py-2 w-full mb-3"
      />
      <button
        onClick={submit}
        disabled={loading}
        className="bg-black text-white px-4 py-2 rounded"
      >
        {loading ? "Queuing..." : "Analyze & Add to Graph"}
      </button>
      {resp && (
        <div className="mt-6 border rounded p-4">
          <div>
            did: <code>{resp.did}</code>
          </div>
          <div>
            jobId: <code>{resp.jobId}</code>
          </div>
          <div>
            status: <b>{resp.status}</b>
          </div>
          <div className="mt-2">
            <a
              className="text-blue-600 underline"
              href={`/user/${encodeURIComponent(resp.did)}`}
            >
              View scores
            </a>
            {resp.status === "done" && (
              <span className="ml-2 text-green-700">ready</span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}



