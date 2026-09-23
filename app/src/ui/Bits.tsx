import type { ReactNode } from "react";

export function Band({ says }: { says: string }) {
  return <div className="band">{says}</div>;
}

export function Line({
  says,
  why,
  more,
  children,
}: {
  says: string;
  why?: ReactNode;
  more?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <div className="line">
      <div className="what">
        <b>{says}</b>
        {why && <span>{why}</span>}
        {more}
      </div>
      <div className="does">{children}</div>
    </div>
  );
}

export function Knob({ on, says, onPress }: { on: boolean; says: string; onPress: () => void }) {
  return (
    <button type="button" className="knob" aria-pressed={on} aria-label={says} onClick={onPress} />
  );
}
