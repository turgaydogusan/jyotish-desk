// Arama kimlikleri (ileri modüller spec'i §12): her harita ve olay araması yeni bir kimlik
// alır. Rust, durdurulan ya da yerine yenisi başlatılan aramanın isteğini bu kimlikle reddeder
// ve durdurulan son kimliği anımsar. Sayaç rastgele bir yerden başlar: pencere yeniden yüklenince
// yeni kimlikler, yüklemeden önce durdurulan ya da hâlâ süren bir aramanınkiyle çakışmaz.
let last = Math.floor(Math.random() * 0x7fff_ffff);

/** Bu oturumda daha önce verilmemiş bir kimlik (u32'ye sığar, 0 değildir). */
export function nextSearchId(): number {
  last = last >= 0xffff_fffe ? 1 : last + 1;
  return last;
}
