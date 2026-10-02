import { Children, cloneElement, isValidElement, type ReactNode } from "react";

export function asProse(parts: string[]): string {
  return parts
    .map((one) => one.trim())
    .filter((one) => one.length > 0)
    .map((one) => (/[.!?:]$/.test(one) ? one : `${one}.`))
    .join(" ");
}

export function Band({ says }: { says: string }) {
  return <div className="band">{says}</div>;
}

export function Line({
  says,
  why,
  more,
  stacked,
  children,
}: {
  says: string;
  why?: ReactNode;
  more?: ReactNode;
  stacked?: boolean;
  children?: ReactNode;
}) {
  const named = `why-${says.replace(/\W+/g, "-").toLowerCase()}`;
  return (
    <div className={stacked ? "line stacked" : "line"}>
      <div className="what">
        <b>{says}</b>
        {why && <span id={named}>{why}</span>}
        {more}
      </div>
      <div className="does">
        {children && <Told named={why ? named : undefined}>{children}</Told>}
      </div>
    </div>
  );
}

function Told({ named, children }: { named?: string; children: ReactNode }) {
  if (!named) return <>{children}</>;
  return (
    <>
      {Children.map(children, (one) =>
        isValidElement<{ "aria-describedby"?: string }>(one)
          ? cloneElement(one, { "aria-describedby": named })
          : one,
      )}
    </>
  );
}

export function Knob({
  on,
  says,
  asleep,
  onPress,
}: {
  on: boolean;
  says: string;
  asleep?: boolean;
  onPress: () => void;
}) {
  return (
    <button
      type="button"
      className="knob"
      role="switch"
      aria-checked={on}
      aria-label={says}
      disabled={asleep}
      onClick={onPress}
    />
  );
}
