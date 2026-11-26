# Yoga el örnekleri: okuma kuralları ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §8 (kural tablosu ve kararlar), §2 (K4 rasi evleri, K5 rasi
çerçevesi, K6 doğal sınıf) ve §4 (burç temelli tam bakış). Bu dizindeki her JSON dosyası bir
sentetik haritadır; `derivation` katalogdaki 41 kuralı sırayla değerlendirir ve `expected.hits`
raporun bütün kayıtlarını yazar. Değerler kuraldan elle türetilmiştir; motor çıktısı buraya
kopyalanmaz (K9). Bir kayıt motorla uyuşmazsa önce kural yeniden okunur.

## Okuma kuralları

Tasarım belgesinin açık bıraktığı yerlerde örnekler şu okumayı kullanır:

1. **Evler ve ilişkiler.** Ev, ilgili burçtan (lagna, Ay, Güneş, Jüpiter ya da Venüs) sayılan
   bütün burç evidir; aynı burç 1. evdir. "Birlikte" aynı burçtur. "Karşılıklı bakış" iki yönlü
   burç temelli tam bakıştır: 7. burç her çiftte karşılıklıdır; özel bakışlardan yalnızca Mars'ın
   4. ve Satürn'ün 10. bakışı birbirini karşılar. "Değişim" iki grahanın birbirinin burcunda
   olmasıdır. Düğümler bakış atmaz, burç lordu değildir ve hiçbir kayıtta yer almaz.
2. **Sayılan grahalar** Mars, Merkür, Jüpiter, Venüs ve Satürn'dür. Kemadruma'da "boş" ve "yalnız"
   bu beş grahaya göre okunur (Güneş ve düğümler Ay'ın yanında durabilir); Ay'dan kendralar 1.,
   4., 7. ve 10. evlerdir.
3. **Mahapurusha** burç düzeyindedir: kendi burcu `own_sign`, yücelme burcu `exaltation` koşulunu
   verir (Merkür Başak'ta ikisini de). Yalnızca lagnadan sayılır.
4. **Lordlar** lagna burcundan sayılan evlerin burç lordlarıdır (Akrep: Mars, Kova: Satürn). Lagna
   lordu hem kendra hem trikona lordudur. Kendra–trikona ve dhana kayıtları çift başınadır; 9. ve
   10. lord çifti iki kayıt verir (kendra–trikona ve Dharma-Karmadhipati). Yogakaraka yalnızca
   lagna burcuna bağlıdır.
5. **Değişim sınıfını** değişen iki ev (grahaların bulunduğu evler) belirler: biri 6, 8 ya da 12
   ise Dainya; değilse biri 3 ise 3. ev lordu değişimi; değilse Maha.
6. **Neecha bhanga** düşüş burcundaki her graha için ayrı kayıttır; düşüş burcun tamamıdır. Koşul
   kimlikleri: `sign_lord_in_kendra`, `exalted_graha_in_kendra`, `dispositor_aspect` (dispozitörün
   burç temelli tam bakışı). Ay kendisinden sayılmaz: lord Ay ise yalnızca lagna referansı
   geçerlidir. Hiçbir koşul sağlanmıyorsa kayıt yoktur.
7. **İki referanslı kurallar** (Amala, Vasumati, Kuja dosha) tek kayıt verir; sağlayan referanslar
   koşuldur (`from_lagna`, `from_moon`, `from_venus`). Amala'da 10. evdeki her graha K6'ya göre
   iyicil olmalıdır.
8. **`grahas` ve `houses`.** `grahas` yogayı oluşturan grahalardır (`Graha::ALL` sırasında); Ay ve
   Güneş yogalarında ışık da listelenir, Neecha bhanga'da yalnızca düşüşteki graha, nabhasa
   yogalarında yedi graha. `houses` lagnadan sayılır: raja ve dhana kayıtlarında çiftin lordu
   olduğu ilgili evler; değişimde değişen iki ev; viparita'da lordu olunan ve bulunulan ev;
   diğerlerinde listelenen grahaların bulunduğu evler.
9. **Doğum saati bilinmiyorsa** (`lagna_lon: null`) lagna isteyen 17 kural değerlendirilmez (beş
   mahapurusha, Kemadruma, Shakata, üç raja, dhana, üç değişim, üç viparita); diğer 24 kural lagna
   seçenekleri olmadan değerlendirilir, `houses` boştur.

`absent` o örneğin "oluşmayan durum" olarak gösterdiği kurallardır. `cancelled` iptal örnekleridir:
temel koşulu sağlandığı hâlde kendradaki grahayla oluşmayan Kemadruma, Ay'ı lagnadan kendrada olan
Shakata ve düşüşte olup hiçbir koşulu sağlanmayan graha (Neecha bhanga kaydı yok).

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan 41 kuralı tasarım belgesinden ve yukarıdaki okuma
kurallarından yeniden değerlendirir; sonucunu `expected` ile karşılaştırır ve aşağıya adını,
tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Plan 18 Task 3 gözden geçiren ajanı (motoru ve örnekleri yazan dışında); son incelemede yeniden koşuldu | 2026-11-26 | 15/15 örnek, 185 kayıt (kimlik, grahalar, evler, ilişkiler, koşullar, sıra, `evaluated`, `skipped_time_unknown`): fark yok. Gözden geçiren 41 kuralı tasarım belgesi §8 ve yukarıdaki okuma kurallarından, başka bir dilde ve kendi tablolarıyla (burç lordları, yücelme ve düşüş burçları, tam bakışlar, K6 sınıfı, burç türleri) yeniden yazdı; her örnekte yalnızca `input`'tan başladı ve motoru çalıştırmadı. `absent` listelerindeki hiçbir kural oluşmuyor; dört iptal örneğinin temel koşulu sağlanıyor. Testlerdeki elle yazılmış yedi tablo Task 1 ve Task 2 gözden geçirmelerinde satır satır yeniden türetildi (fark yok). Sınır: gözden geçiren aynı görevde motorun kaynağını da inceledi ve aynı okuma kurallarını paylaşır; bu kayıt kuralların uygulanışını doğrular, okuma seçimlerini (tasarım belgesi §17) değil. |
