import { reactive } from "vue";

export type CtxItem =
  | {
      type?: "item";
      label: string;
      hint?: string;
      accent?: boolean;
      disabled?: boolean;
      run: () => void;
    }
  | { type: "sep" };

const state = reactive({
  open: false,
  x: 0,
  y: 0,
  items: [] as CtxItem[],
});

export function contextMenuState() {
  return state;
}

export function showContextMenu(e: MouseEvent, items: CtxItem[]) {
  e.preventDefault();
  e.stopPropagation();
  if (!items.length) return;
  state.x = e.clientX;
  state.y = e.clientY;
  state.items = items;
  state.open = true;
}

export function hideContextMenu() {
  state.open = false;
}

export function isEditableTarget(target: EventTarget | null): target is HTMLInputElement | HTMLTextAreaElement {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement) {
    return !["button", "submit", "checkbox", "radio", "file", "range"].includes(target.type);
  }
  return target instanceof HTMLTextAreaElement || target.isContentEditable;
}

export function editMenuItems(el: HTMLInputElement | HTMLTextAreaElement): CtxItem[] {
  const run = (fn: () => void) => () => {
    el.focus();
    fn();
  };
  return [
    {
      label: "Вырезать",
      run: run(() => document.execCommand("cut")),
    },
    {
      label: "Копировать",
      run: run(() => document.execCommand("copy")),
    },
    {
      label: "Вставить",
      run: run(async () => {
        try {
          const text = await navigator.clipboard.readText();
          const start = el.selectionStart ?? el.value.length;
          const end = el.selectionEnd ?? el.value.length;
          const next = el.value.slice(0, start) + text + el.value.slice(end);
          el.value = next;
          const pos = start + text.length;
          el.setSelectionRange(pos, pos);
          el.dispatchEvent(new Event("input", { bubbles: true }));
        } catch {
          document.execCommand("paste");
        }
      }),
    },
    { type: "sep" },
    {
      label: "Выделить всё",
      run: run(() => el.select()),
    },
  ];
}
