import { useCallback, useEffect, useRef, useState } from "react";
import { deleteGoal, getGoal, updateGoal } from "../api";
import { assetUrl } from "../lib/assets";
import type { Goal } from "../types";
import CoverImage from "../ui/CoverImage";
import InlineText from "../ui/InlineText";
import Mark from "../ui/Mark";
import Skeleton from "../ui/Skeleton";
import TwoClickDelete from "../ui/TwoClickDelete";
import CoverPicker, { CoverCredit } from "./CoverPicker";

interface Props {
  goalId: string;
  libraryRoot: string;
  onDeleted: () => void;
  onChanged: () => void;
  onError: (message: string | null) => void;
}

export default function GoalView({ goalId, libraryRoot, onDeleted, onChanged, onError }: Props) {
  const [goal, setGoal] = useState<Goal | null>(null);
  const [notes, setNotes] = useState("");
  const [picker, setPicker] = useState(false);
  const [adjusting, setAdjusting] = useState(false);
  const notesTimer = useRef<number | null>(null);
  const notesRef = useRef(notes);
  const notesHydratedFor = useRef<string | null>(null);
  notesRef.current = notes;

  const refresh = useCallback(async () => {
    const next = await getGoal(goalId);
    setGoal(next);
    if (notesHydratedFor.current !== goalId) {
      setNotes(next.notes ?? "");
      notesHydratedFor.current = goalId;
    }
  }, [goalId]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  const saveNotes = useCallback(
    async (value: string) => {
      try {
        const next = await updateGoal(goalId, { notes: value });
        setGoal(next);
      } catch (err) {
        onError(String(err));
      }
    },
    [goalId, onError],
  );

  if (!goal) return <Skeleton count={3} />;

  const done = goal.status === "done";

  return (
    <div className="goal-page">
      <div className="goal-head">
        <div className="cover-band">
          {goal.cover_relpath ? (
            <CoverImage
              src={assetUrl(libraryRoot, goal.cover_relpath)}
              focusX={goal.cover_x}
              focusY={goal.cover_y}
            />
          ) : (
            <div className="cover-empty">
              <button type="button" className="text" onClick={() => setPicker(true)}>
                Choose a cover
              </button>
            </div>
          )}
        </div>
        <div className="goal-intro">
          <div className="title-row">
            <InlineText
              as="h1"
              className="page-title"
              value={goal.name}
              onSave={async (name) => {
                const next = await updateGoal(goalId, { name });
                setGoal(next);
                onChanged();
              }}
            />
            <Mark
              been={done}
              onLabel="done"
              onToggle={() => {
                updateGoal(goalId, { status: done ? "dream" : "done" })
                  .then((next) => {
                    setGoal(next);
                    onChanged();
                  })
                  .catch((err) => onError(String(err)));
              }}
            />
          </div>
          {goal.cover_credit && (
            <p className="cover-credit">
              <CoverCredit credit={goal.cover_credit} url={goal.cover_credit_url} />
            </p>
          )}
          <div className="page-actions">
            <button type="button" className="text" onClick={() => setPicker(true)}>
              Cover
            </button>
            {goal.cover_relpath && (
              <button
                type="button"
                className="text"
                onClick={() => {
                  setAdjusting(true);
                  setPicker(true);
                }}
              >
                Move
              </button>
            )}
            <TwoClickDelete
              label={goal.name}
              confirmLabel={`Delete ${goal.name}?`}
              onConfirm={async () => {
                await deleteGoal(goalId);
                onDeleted();
              }}
            />
          </div>
          <textarea
            className="notes"
            value={notes}
            placeholder="Notes"
            rows={4}
            onChange={(e) => {
              const value = e.target.value;
              setNotes(value);
              if (notesTimer.current != null) window.clearTimeout(notesTimer.current);
              notesTimer.current = window.setTimeout(() => {
                void saveNotes(value);
              }, 600);
            }}
            onBlur={() => {
              if (notesTimer.current != null) {
                window.clearTimeout(notesTimer.current);
                notesTimer.current = null;
              }
              void saveNotes(notesRef.current);
            }}
          />
        </div>
      </div>

      {picker && (
        <CoverPicker
          libraryRoot={libraryRoot}
          initialQuery={goal.name}
          target="goal"
          ownerId={goalId}
          shape="wide"
          current={
            goal.cover_relpath
              ? {
                  src: assetUrl(libraryRoot, goal.cover_relpath),
                  x: goal.cover_x,
                  y: goal.cover_y,
                }
              : null
          }
          startOnCurrent={adjusting}
          onClose={() => {
            setPicker(false);
            setAdjusting(false);
          }}
          onApplied={async () => {
            await refresh();
            onChanged();
          }}
          onError={onError}
        />
      )}
    </div>
  );
}
