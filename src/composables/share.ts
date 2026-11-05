import { reactive } from "vue";

export const shareDialog = reactive({
  open: false,
  path: "",
});

export function askShare(path: string) {
  shareDialog.path = path;
  shareDialog.open = true;
}
