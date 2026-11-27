// Arama kimlikleri (ileri modüller spec'i §12): her harita ve olay araması yeni bir kimlik
// alır. Rust, durdurulan ya da yerine yenisi başlatılan aramanın isteğini bu kimlikle reddeder.
let last = 0;

/** Bu oturumda daha önce verilmemiş bir kimlik (u32'ye sığar). */
export function nextSearchId(): number {
  last = last >= 0xffff_fffe ? 1 : last + 1;
  return last;
}
