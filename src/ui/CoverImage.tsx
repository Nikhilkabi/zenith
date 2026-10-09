import { useCallback, useEffect, useRef, useState } from "react";
import { objectPosition } from "../lib/coverFocus";

interface Props {
  src: string;
  focusX: number;
  focusY: number;
  loading?: "lazy" | "eager";
}

export default function CoverImage({ src, focusX, focusY, loading = "lazy" }: Props) {
  const ref = useRef<HTMLImageElement>(null);
  const [position, setPosition] = useState("50% 50%");

  const place = useCallback(() => {
    const img = ref.current;
    const frame = img?.parentElement;
    if (!img || !frame || img.naturalWidth < 1) return;
    setPosition(
      objectPosition(
        img.naturalWidth,
        img.naturalHeight,
        frame.clientWidth,
        frame.clientHeight,
        focusX,
        focusY,
      ),
    );
  }, [focusX, focusY]);

  useEffect(() => {
    place();
  }, [place, src]);

  useEffect(() => {
    const frame = ref.current?.parentElement;
    if (!frame || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(() => place());
    observer.observe(frame);
    return () => observer.disconnect();
  }, [place]);

  return (
    <img
      ref={ref}
      src={src}
      alt=""
      loading={loading}
      draggable={false}
      style={{ objectPosition: position }}
      onLoad={place}
    />
  );
}
