export function TokenGroups({ groups }) {
  if (!groups.length) {
    return <p className="docs-empty">No tokens in this family.</p>;
  }
  return (
    <div className="type-tables">
      {groups.map((group) => (
        <TokenTable key={group.id} title={group.name} rows={group.rows || []} />
      ))}
    </div>
  );
}

export function TokenTable({ title, rows }) {
  return (
    <section className="token-table-wrap">
      <h3 className="token-table-title">{title}</h3>
      <table className="token-table">
        <thead>
          <tr>
            <th>Token</th>
            <th>Value</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.token}>
              <td>{row.token}</td>
              <td>{row.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
