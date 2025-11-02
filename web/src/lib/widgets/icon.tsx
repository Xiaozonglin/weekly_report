import { JSX } from "solid-js";

type IconProps = {
    name: string; // semantic name like 'refresh' or full class like 'fluent--copy-20-regular'
    class?: string;
    ariaHidden?: boolean;
};

export default function Icon(props: IconProps) {
    const cls = props.class || "w-5 h-5";
    const ariaHidden = props.ariaHidden ?? true;

    // mapping semantic names to either Tailwind Iconify class or inline SVG
    const map: Record<string, JSX.Element | string> = {
        // semantic -> class (most cases) or SVG (fallback)
        "copy": "icon-[fluent--copy-20-regular]",
        "check": "icon-[fluent--checkmark-20-regular]",
        "settings": "icon-[fluent--settings-20-regular]",
        // prefer fluent icon for refresh; fall back to inline svg if not available
        "refresh": "icon-[fluent--sync-20-regular]",
    };

    const val = map[props.name];
    if (!val) {
        // if name looks like fluent class part, try to render as class-based icon
        if (props.name.includes("fluent--") || props.name.startsWith("fluent--") || props.name.includes("-20-")) {
            // allow passing full class like "fluent--copy-20-regular" or "icon-[fluent--copy-20-regular]"
            const className = props.name.includes("icon-") ? props.name : `icon-[${props.name}]`;
            return <span class={`${className} ${cls}`} aria-hidden={ariaHidden} />;
        }
        // fallback: render name as fluent with default suffix
        const fallback = `icon-[fluent--${props.name}-20-regular]`;
        return <span class={`${fallback} ${cls}`} aria-hidden={ariaHidden} />;
    }

    if (typeof val === "string") {
        // treat as class name
        const className = val.includes("icon-") ? val : `icon-[${val}]`;
        return <span class={`${className} ${cls}`} aria-hidden={ariaHidden} />;
    }

    // val is JSX.Element (inline svg)
    return val;
}
