async function getScores(id: string) {
  const base = process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8080";
  const r = await fetch(`${base}/v1/user/${id}/scores`, { cache: "no-store" });
  return r.json();
}

export default async function UserPage({ params }: { params: { id: string } }) {
  const data = await getScores(params.id);
  return (
    <div className="p-6 max-w-3xl mx-auto space-y-4">
      <h1 className="text-2xl font-semibold">Scores for {data.handle || data.did}</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div className="border rounded p-3">
          <h3 className="font-medium mb-1">Accuracy</h3>
          <pre className="text-sm">{JSON.stringify(data.facets?.accuracy, null, 2)}</pre>
        </div>
        <div className="border rounded p-3">
          <h3 className="font-medium mb-1">Civility</h3>
          <pre className="text-sm">{JSON.stringify(data.facets?.civility, null, 2)}</pre>
        </div>
        <div className="border rounded p-3">
          <h3 className="font-medium mb-1">Bot Probability</h3>
          <div className="text-xl">{Math.round((data.botProb || 0) * 100)}%</div>
        </div>
      </div>
      <div className="border rounded p-3">
        <h3 className="font-medium mb-2">Domain Expertise</h3>
        <ul className="list-disc pl-5">
          {(data.expertise || []).map((e: any) => (
            <li key={e.domain}>
              {e.domain}: {Math.round(e.score * 100) / 100}
            </li>
          ))}
        </ul>
      </div>
      <div className="border rounded p-3">
        <h3 className="font-medium mb-2">Evidence</h3>
        <pre className="text-sm">{JSON.stringify(data.evidence || [], null, 2)}</pre>
      </div>
    </div>
  );
}



