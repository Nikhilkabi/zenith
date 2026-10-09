import { useCallback, useEffect, useState } from "react";
import { listTrash, purgeTrash, restoreTrash } from "../api";
import type { TrashItem } from "../types";
import Skeleton from "../ui/Skeleton";

interface Props {
  onChanged: () => void;
  onError: (message: string | null) => void;
}

export default function Trash({ onChanged, onError }: Props) {
  const [items, setItems] = useState<TrashItem[] | null>(null);

  const refresh = useCallback(async () => {
    setItems(await listTrash());
  }, []);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  if (!items) return <Skeleton count={3} />;

  return (
    <div>
      <div className="page-header">
        <div>
          <h1 className="page-title">Trash</h1>
          <p className="meta">Restored items come back. Files leave after 30 days.</p>
        </div>
        <div className="page-actions">
          <button
            type="button"
            className="text"
            disabled={items.length === 0}
            onClick={() => {
              purgeTrash()
                .then(async (n) => {
                  await refresh();
                  onChanged();
                  onError(n > 0 ? null : null);
                })
                .catch((err) => onError(String(err)));
            }}
          >
            Purge old
          </button>
        </div>
      </div>

      {items.length === 0 ? (
        <div className="empty-square">Empty</div>
      ) : (
        <ul className="trash-list">
          {items.map((item) => (
            <li key={`${item.kind}-${item.id}`}>
              <div>
                <div className="title">
                  {item.kind} · {item.title}
                </div>
                <div className="sub" title={item.deleted_at}>
                  {new Date(item.deleted_at).toLocaleString()}
                </div>
              </div>
              <button
                type="button"
                className="text"
                onClick={() => {
                  restoreTrash(item.kind, item.id)
                    .then(async () => {
                      await refresh();
                      onChanged();
                      onError(null);
                    })
                    .catch((err) => onError(String(err)));
                }}
              >
                Restore
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
