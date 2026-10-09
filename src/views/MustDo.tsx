import { useCallback, useEffect, useState } from "react";
import { addPlaceTask, deletePlaceTask, listPlaceTasks, setPlaceTask } from "../api";
import type { PlaceTask } from "../types";

interface Props {
  placeId: string;
  onError: (message: string | null) => void;
}

export default function MustDo({ placeId, onError }: Props) {
  const [tasks, setTasks] = useState<PlaceTask[]>([]);
  const [draft, setDraft] = useState("");

  const refresh = useCallback(async () => {
    setTasks(await listPlaceTasks(placeId));
  }, [placeId]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  return (
    <section className="trip-block">
      <h2 className="section-title">Must-do</h2>
      {tasks.length === 0 ? (
        <p className="links-empty">A short list for this place. Sunrise viewpoint, the trail, the village.</p>
      ) : (
        <ul className="task-list">
          {tasks.map((task) => (
            <li key={task.id}>
              <label>
                <input
                  type="checkbox"
                  checked={task.done}
                  onChange={() => {
                    setPlaceTask(task.id, { done: !task.done })
                      .then((next) =>
                        setTasks((current) => current.map((item) => (item.id === next.id ? next : item))),
                      )
                      .catch((err) => onError(String(err)));
                  }}
                />
                <span className={task.done ? "done" : undefined}>{task.body}</span>
              </label>
              <button
                type="button"
                className="text"
                onClick={() => {
                  deletePlaceTask(task.id)
                    .then(() => setTasks((current) => current.filter((item) => item.id !== task.id)))
                    .catch((err) => onError(String(err)));
                }}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}
      <form
        className="task-add"
        onSubmit={(event) => {
          event.preventDefault();
          const body = draft.trim();
          if (!body) return;
          addPlaceTask(placeId, body)
            .then((task) => {
              setTasks((current) => [...current, task]);
              setDraft("");
              onError(null);
            })
            .catch((err) => onError(String(err)));
        }}
      >
        <input
          value={draft}
          placeholder="Add something to do here"
          onChange={(event) => setDraft(event.target.value)}
        />
      </form>
    </section>
  );
}
