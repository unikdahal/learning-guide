# Emerging-technology cases: evidence record

Checked 17 November 2026. Lesson: `src/content/docs/lok-sewa/preparation/06-emerging-technology-cases.mdx`. All cases, figures and the ten practice questions are original. Nothing here is a past paper, an official sample, a predicted question or a topic-frequency or marks claim.

## Primary documents fetched (sha256 of the copy fetched this session)

| Document | Used for | sha256 |
| --- | --- | --- |
| [PSC CE Paper II](https://psc.gov.np/site_uploads/course/20231110070747654dd6c3886821.pdf), 6 pp. | Unit 15 list (p. 4); 1.6 "Distributed system, Clusters" (p. 1); 10.5 "Distributed Systems: Distributed Message passing, RPC, Client/Server Computing, Clusters" (p. 3), Section C = units 11–15, 20% | `bb82b794…d5f808e` |
| [PSC CO Paper II](https://psc.gov.np/site_uploads/course/20231110070338654dd5cad73731.pdf) | CO overlap lines only (below) | `147792f4…2d46fed` |
| [NEC ACtE syllabus](https://nec.gov.np/uploads/Syllabus-Registration/1763716654201_Syllabus%20Computer%20Engineering.pdf), 6 pp. | Partial-overlap check | `3b0edb04…14fc76` |
| [OMG CORBA 3.3 Part 2](https://www.omg.org/spec/CORBA/3.3/Interoperability/PDF), 206 pp. | CDR: the sender sets the byte order and a flag indicates it; primitives aligned on natural boundaries; encapsulation byte-order flag | `6872758c…e050c` |
| RFC 5531 | §5: a reply over a retrying unreliable transport implies at least once; xid duplicate cache gives "some degree of execute-at-most-once". §9: rpc_msg/call_body fields; opaque_auth = flavor + opaque body. §11: 4-byte record mark, high bit = last fragment | rfc-editor.org text |
| RFC 4506 | §4.1: 32-bit big-endian int. §4.11: string = length + bytes + 0–3 zero pad to a multiple of 4 | rfc-editor.org text |
| RFC 7946 | §3.1.1: first two elements are "longitude and latitude, or easting and northing, precisely in that order" | rfc-editor.org text |
| RFC 9110 | §9.2.2: PUT, DELETE and safe methods are idempotent | rfc-editor.org text |
| RFC 2104 | HMAC (named as an example of a callback MAC only) | rfc-editor.org text |
| [MS-DCOM Reference Counts](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dcom/69771298-e621-4b37-85fe-f97f8dad4974) | Per-interface reference counts (public and private); keep-alive pinging; a server reclaims an object when pings stop | Microsoft Learn page |
| [MS-DCOM Object Resolver Service](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dcom/3df23440-54ed-40f5-ba86-4a848918202f) | Activation, OXID resolution, garbage collection and aliveness through RPC interfaces | Microsoft Learn page |

Full hashes are recorded below so another session can confirm it fetched the same files.

- CE `bb82b794cea7db04cc86fe5b20d0b5521c8eceb21a9428fd819aee4e1f5d808e`
- CO `147792f451dd1b09bf5b3c5b75c69bcbe385abcfdfbea60ffa97a1bcc2d46fed`
- NEC `3b0edb0438684598b901c963f23fefddf695a30d0e0c2db5537f7e84bf14fc76`
- CORBA Part 2 `6872758c9fe01a756cd3b5af2764169231df12b290337191c74f0c8def0e050c`

## Scope tags and overlap (exact printed labels)

- **CE tags used:** 15.2, 15.6, 15.7 "Distributed Object technology ( ORB, DCOM)", 15.8, 15.10, 15.13, 15.14 and 15.16 "GIS/ Remote sensing/ GPS".
  - RPC is printed under CE 10.5 and clusters under CE 1.6. The lesson tags RPC as "15.7 (with CE 10.5)".
  - E-commerce security is tagged "15.10 (with CE 15.9)" because cryptography and network security are 15.9.
- **CO overlap, cited only where printed:**
  - 5.5 "Concepts of Parallel and Distributed Processing"
  - 10.3 "Electronic Commerce Security Issues"
  - 10.5 "E-Business: b2b, b2c, b2e, c2c, g2g, g2c"
  - 10.6 "Principles of Electronic Payment, Strategies & Systems"
  - 11.1 "Information Systems, Client-Server Computing"
  - 11.7 "Multimedia"
- **CO 11.8 is not claimed.** It reads "Object-Oriented Programming with COMS & DECOMS". Reading that as COM/DCOM is an editorial guess, so the lesson does not claim it as an overlap.
- **NEC, partial only:**
  - ACtE0401: "Pipelining parallel processing"
  - ACtE0505: sockets, application server
  - ACtE0506: SSL, digital signatures
  - ACtE0802: "Client –server architectures, Distributed object architectures"
- **Not listed in the NEC ACtE syllabus** (checked by text search of the fetched PDF, not of the repo's NEC lessons): GIS, remote sensing, GPS, image processing, speech, multimedia, e-commerce, ORB or DCOM by name. The lesson says NEC preparation alone does not cover these.

## Verified numeric traces

The script is reproduced in full under [Verification script](#verification-script). It uses Python `fractions` where results are exact. Run it with `python3 -I verify.py`: expect 62 lines beginning `OK` and a final line `ALL CHECKS PASSED`.

### Case 1: speedup and clocks

- **Amdahl, f = 0.9:**
  - S(8) = 80/17 ≈ 4.706 and E ≈ 0.588. The limit is 10.
  - The smallest p with S ≥ 5 is 9, and S(9) = 5 exactly.
- **Overhead model** T = 0.1 + 0.9/p + 0.01(p − 1):
  - S(4) = 2.817, S(8) = 3.540 and S(16) = 3.265.
  - The maximum over p = 1…40 is at p ∈ {9, 10}, both 25/7.
- **Gustafson:** s = 0.1 (of the p-run), p = 8 gives 7.3.
- **Lamport/vector clocks, events a b e c d f g h:**
  - Lamport values a…h = 1, 2, 3, 4, 1, 5, 6, 7.
  - Vectors as in the lesson table. e is concurrent with exactly a, b, c and d (Q2), and e → f.

### Case 2: wire bytes

- XDR `(42, "Kathmandu")` = `00 00 00 2a 00 00 00 09 4b 61 74 68 6d 61 6e 64 75 00 00 00`, which is 20 bytes.
- The AUTH_NONE call header is 6 × 4 + 8 + 8 = 40 bytes, so the call is 60 bytes and 64 with the record mark `8000003c`.
- Q3: `(7, "Pokhara")` gives 16, 56 and 60 bytes.

### Case 3: raster, coordinates and ranging

- **Raster size:**
  - 3000 × 2000 cells = 6,000,000; with 4 bands × 2 bytes that is 48,000,000 bytes. At 5 m it is 192,000,000.
  - 2¹² = 4096 levels.
  - Q6: 1,400,000 bytes, then 5,600,000.
- **Affine, corner origin (500000, 3100000), d = 10, north-up:**
  - The centre of (120, 250) is (502505, 3098795).
  - (503012, 3097333) → row 266, col 301.
  - Q7: (501234, 3099876) → row 12, col 123, centre (501235, 3099875).
- **Ranging:**
  - Trilateration toy: squared ranges 25, 65 and 45 → (3, 4).
  - c × 1 µs = 299.792458 m.

### Case 4: media

- **Image:**
  - 1920 × 1080 × 3 = 6,220,800 bytes, taking 4.977 s at 10⁷ bit/s. At a 10:1 ratio it is 622,080 bytes.
  - Histogram equalisation: counts {2: 4, 3: 5, 4: 5, 5: 2}; 7 × CDF = 1.75, 3.9375, 6.125 and 7. The map 2→2, 3→4, 4→6, 5→7 is round-to-nearest, and the script asserts there are no .5 ties. Row 3 becomes 2 6 7 6.
  - Filters: mean 346/9, median 12. Q8: median 50, mean 608/9.
- **Speech:**
  - Window/hop 400/160 gives 98 frames, with no padding.
  - 256 kbit/s; energy 45; zero crossings 3 (with 0 counted as non-negative).
  - Q9: 199 frames and 64 kbit/s.
- **Multimedia:**
  - 1,411,200 bit/s, which over 180 s is 31,752,000 bytes.
  - 720p25 at 24-bit = 552,960,000 bit/s.
  - Decode order I1 P4 B2 B3 P7 B5 B6.

### Cases 5–6: shop and availability

- **Price recomputation:** the subtotal is 2340 and the claimed 1650 is rejected. With a hypothetical 10% tax the total is 2574.
- **Availability:**
  - Serial 0.999 × 0.999 × 0.995 = 0.993010995, about 61.2 h a year.
  - With a database pair at 0.999975 the total is ≈ 0.997976 (about 17.7 h).
  - Q10: 0.9999 × 0.995 = 0.9949005.

## Assumptions stated in the lesson

- **Amdahl:** fixed problem size, perfect balance, no communication cost unless the stated overhead term is added. f is a fraction of the one-processor time.
- **Gustafson:** s is a fraction of the p-processor time, which is a different base from Amdahl's.
- **Lamport ties:** broken by process id only when a total order is needed.
- **Byte counts:** exclude IP and TCP headers and use AUTH_NONE. Decimal prefixes are used throughout, and protocol overhead is ignored for media transfers.
- **Compression ratio** is original : compressed. The 10:1 ratio is a hypothetical encoder output, not a property of any format.
- **Raster:**
  - 12-bit data is stored in 16-bit containers.
  - The grid is north-up, uses a corner origin with zero-based (row, col), and N decreases with the row index.
  - GeoJSON order is longitude, latitude.
- **Histogram equalisation:** round to nearest; floor changes the result, and the lesson says so.
- **Speech:** no padding at the end; zero is treated as non-negative for crossings.
- **Availability:** failures are independent, and the lesson flags correlated failure as the limit of that assumption.
- **Tax:** the 10% rate is explicitly hypothetical. No Nepal policy rates are used.

## Deliberately excluded (volatile or unverifiable)

The lesson leaves out:

- GNSS constellation sizes and signal frequencies
- TLS or PCI DSS version numbers
- OWASP rankings
- DCOM hardening timelines
- codec, product or vendor recommendations
- Nepal tax or payment-regulation figures

ATAM, the voiced/unvoiced heuristic and the Gonzalez/Woods filter facts rest on the named textbooks. Those books were not fetched in this session. Naming a text is not a claim that every edition was opened.

## Coverage boundaries

- **Deepened here:** 15.2, 15.6, 15.7, 15.8, 15.10, 15.13, 15.14 and 15.16.
- **Not deepened here:**
  - 15.3, 15.4, 15.5, 15.9, 15.11, 15.12 and 15.15. They remain at summary depth in `ce-gap-units.mdx`, or in NEC lessons where those exist.
  - 15.1 is already worked in `ce-gap-units.mdx`.
- **Selected cases, not complete courses.** For example, the lesson does not cover:
  - image transforms (FFT/DCT), segmentation or morphology
  - MFCC computation, HMM recognition
  - MPEG motion estimation
  - GIS projections and datums, spatial analysis operators
  - CORBA services or POA policies in detail
  - Java RMI
  - e-commerce business strategy

## Checks run

- **MDX compile:** `@mdx-js/mdx` with `remark-math` and `rehype-katex` (`output: 'mathml'`, `strict: 'error'`) from the repo's `node_modules`, run from the scratchpad. It compiled with 14 TeX annotations and 0 `katex-error`.
- **Static scans:** no HTML comments and no stray braces outside the import line, math and code. The file is 382 lines.
- **Anchors:** both internal anchors match existing headings (`ce-gap-units` "Other unit 15 electives at 5-mark depth"; `co-non-overlap` "E-commerce and MIS (unit 10 and 11)").
- **Figures:** three inline SVGs with `role="img"`, a descriptive `aria-label` and theme variables. Marker ids are unique (`emt-ah1`–`emt-ah3`).
- **Not run:**
  - The site build, `npm run verify` and the browser tests (shared checkout). The parent must run them, including the 375px overflow check.
  - Adding the page to the browser-test lesson list.
- **Process note:** a temporary compile helper was briefly copied into the repo root during checking and deleted immediately; it is not in git status.

## Verification script

```python
"""Verify every numeric trace in 06-emerging-technology-cases.mdx. Run: python3 -I verify.py"""
from fractions import Fraction as F
from collections import Counter
import math, struct

def ok(name, got, want):
    assert got == want, (name, got, want)
    print(f"OK  {name}: {got}")

# --- CE 15.2 Amdahl / Gustafson -------------------------------------------
f = F(9, 10)  # parallelisable fraction of the ONE-processor run time
def amdahl(f, p): return 1 / ((1 - f) + f / p)
S8 = amdahl(f, 8)
ok("Amdahl S(8) exact", S8, F(80, 17))
print("    S(8) ~", float(S8), " E(8) ~", float(S8 / 8))
ok("Amdahl limit", 1 / (1 - f), F(10))
# processors needed for S = 5
p_needed = [p for p in range(1, 100) if amdahl(f, p) >= 5][0]
ok("p for S>=5", p_needed, 9)
ok("S(9) exactly 5", amdahl(f, 9), F(5))
# with overhead 0.01*(p-1) of serial time added (stated assumption)
def amdahl_ovh(f, p, c): return 1 / ((1 - f) + f / p + c * (p - 1))
for p in (4, 8, 16):
    print(f"    overhead model p={p}: S={float(amdahl_ovh(f,p,F(1,100))):.3f}")
ok("overhead S(8)", amdahl_ovh(f, 8, F(1, 100)), 1 / (F(1,10) + F(9,80) + F(7,100)))
top = max(amdahl_ovh(f, p, F(1, 100)) for p in range(1, 41))
ok("overhead best p set", [p for p in range(1, 41) if amdahl_ovh(f, p, F(1, 100)) == top], [9, 10])
ok("overhead peak", top, F(25, 7))
print("    S(9), S(10) with overhead:", float(amdahl_ovh(f,9,F(1,100))), float(amdahl_ovh(f,10,F(1,100))))
# Gustafson: serial fraction s measured on the p-processor run
s = F(1, 10)
ok("Gustafson scaled S(8)", s + 8 * (1 - s), F(73, 10))

# --- CE 10.5/15.2 Lamport and vector clocks -------------------------------
# events: (proc, kind, msg)
events = [
    (0, "local", None),        # a
    (0, "send", "m1"),         # b  P1 -> P2
    (2, "local", None),        # e
    (1, "recv", "m1"),         # c
    (1, "send", "m2"),         # d  P2 -> P3
    (2, "recv", "m2"),         # f
    (2, "send", "m3"),         # g  P3 -> P1
    (0, "recv", "m3"),         # h
]
names = "a b e c d f g h".split()
L = [0, 0, 0]; V = [[0]*3 for _ in range(3)]; msgL = {}; msgV = {}; res = {}
for nm, (p, k, m) in zip(names, events):
    if k == "recv":
        L[p] = max(L[p], msgL[m]) + 1
        V[p] = [max(x, y) for x, y in zip(V[p], msgV[m])]; V[p][p] += 1
    else:
        L[p] += 1; V[p][p] += 1
        if k == "send": msgL[m] = L[p]; msgV[m] = V[p][:]
    res[nm] = (L[p], tuple(V[p]))
for nm in "abcdefgh": print("   ", nm, res[nm])
ok("Lamport a..h", [res[x][0] for x in "abcdefgh"], [1, 2, 3, 4, 1, 5, 6, 7])
def hb(u, v): return all(x <= y for x, y in zip(u, v)) and u != v
ok("e vs c concurrent", (hb(res['e'][1], res['c'][1]), hb(res['c'][1], res['e'][1])), (False, False))
ok("e -> f", hb(res['e'][1], res['f'][1]), True)

# --- CE 15.7 / 10.5 XDR + ONC RPC bytes -----------------------------------
def xdr_int(v): return struct.pack(">i", v)
def xdr_string(s):
    b = s.encode("ascii"); pad = (-len(b)) % 4
    return struct.pack(">I", len(b)) + b + b"\x00" * pad
args = xdr_int(42) + xdr_string("Kathmandu")
print("    args hex:", args.hex(" "))
ok("XDR args bytes", len(args), 20)
# RFC 5531: xid, mtype, rpcvers, prog, vers, proc = 6 words; cred & verf AUTH_NONE = flavor + zero length = 8 each
hdr = 6 * 4 + 8 + 8
ok("ONC RPC call header (AUTH_NONE)", hdr, 40)
ok("call message", hdr + len(args), 60)
ok("with TCP record mark", hdr + len(args) + 4, 64)
ok("record mark word", (0x80000000 | 60).to_bytes(4, "big").hex(), "8000003c")
ok("XDR 'Pokhara' string", len(xdr_string("Pokhara")), 12)

# --- CE 15.16 raster, affine, GPS ------------------------------------------
cols, rows = 30_000 // 10, 20_000 // 10
ok("pixels", cols * rows, 6_000_000)
ok("bytes 4 bands 16-bit containers", cols * rows * 4 * 2, 48_000_000)
ok("bytes at 5 m", (30_000 // 5) * (20_000 // 5) * 4 * 2, 192_000_000)
ok("12-bit levels", 2 ** 12, 4096)
X0, Y0, d = 500_000, 3_100_000, 10
def centre(r, c): return (X0 + (c + F(1, 2)) * d, Y0 - (r + F(1, 2)) * d)
ok("centre(120,250)", centre(120, 250), (502_505, 3_098_795))
def to_rc(E, N): return (math.floor(F(Y0 - N, d)), math.floor(F(E - X0, d)))
ok("pixel of (503012, 3097333)", to_rc(503_012, 3_097_333), (266, 301))
c = 299_792_458
ok("1 us range error (m)", F(c, 10**6), F(299792458, 10**6))
print("    1 us ->", c / 1e6, "m")
# 2-D trilateration toy
A, B, C, P = (0, 0), (10, 0), (0, 10), (3, 4)
r2 = [ (P[0]-q[0])**2 + (P[1]-q[1])**2 for q in (A, B, C)]
ok("squared ranges", r2, [25, 65, 45])
x = F(r2[0] - r2[1] + 100, 20); y = F(r2[0] - r2[2] + 100, 20)
ok("trilateration", (x, y), (3, 4))
print("    ranges:", [round(math.sqrt(v), 4) for v in r2])

# --- CE 15.13 image --------------------------------------------------------
raw = 1920 * 1080 * 3
ok("raw RGB bytes", raw, 6_220_800)
ok("10 Mbit/s time", F(raw * 8, 10_000_000), F(497664, 100000))
ok("10:1 compressed bytes", F(raw, 10), 622_080)
img = [[2, 3, 3, 2], [3, 4, 4, 3], [2, 4, 5, 4], [3, 4, 5, 2]]
flat = [v for r in img for v in r]; cnt = Counter(flat); n = len(flat); Lv = 8
cum = 0; mp = {}
for k in range(Lv):
    cum += cnt.get(k, 0)
    sv = F(Lv - 1) * F(cum, n)
    if cnt.get(k, 0):
        assert sv.denominator == 1 or (sv - math.floor(sv)) != F(1, 2), "tie"
        mp[k] = math.floor(sv + F(1, 2))
        print(f"    r={k} n={cnt[k]} cum={cum} 7*cdf={sv} ({float(sv)}) -> {mp[k]}")
ok("counts", dict(sorted(cnt.items())), {2: 4, 3: 5, 4: 5, 5: 2})
ok("HE map", mp, {2: 2, 3: 4, 4: 6, 5: 7})
ok("equalised row 3", [mp[v] for v in img[2]], [2, 6, 7, 6])
nb = [10, 12, 11, 13, 255, 12, 11, 10, 12]
ok("mean 3x3", F(sum(nb), 9), F(346, 9)); print("    mean ~", sum(nb) / 9)
ok("median 3x3", sorted(nb)[4], 12)

# --- CE 15.8 speech --------------------------------------------------------
fs = 16_000; win = fs * 25 // 1000; hop = fs * 10 // 1000
ok("win/hop samples", (win, hop), (400, 160))
ok("frames in 1 s, no padding", (fs - win) // hop + 1, 98)
ok("16-bit mono bitrate", fs * 16, 256_000)
x = [3, 1, -2, -4, -1, 2, 3, -1]
ok("short-time energy", sum(v * v for v in x), 45)
zc = sum(1 for a, b in zip(x, x[1:]) if (a >= 0) != (b >= 0))
ok("zero crossings", zc, 3)

# --- CE 15.14 multimedia ---------------------------------------------------
ok("CD audio bit/s", 44_100 * 16 * 2, 1_411_200)
ok("3 min CD bytes", 44_100 * 16 * 2 * 180 // 8, 31_752_000)
ok("720p25 raw bit/s", 1280 * 720 * 24 * 25, 552_960_000)
display = ["I1", "B2", "B3", "P4", "B5", "B6", "P7"]
# decode order: each anchor (I/P) before the B frames that precede it in display order
dec, pend = [], []
for fr in display:
    if fr[0] == "B": pend.append(fr)
    else: dec.append(fr); dec += pend; pend = []
dec += pend
ok("decode order", dec, ["I1", "P4", "B2", "B3", "P7", "B5", "B6"])

# --- CE 15.10 e-commerce ---------------------------------------------------
cart = [(2, 450), (1, 1200), (3, 80)]   # qty, server-side unit price (paisa-free rupees)
sub = sum(q * p for q, p in cart)
ok("subtotal", sub, 2340)
ok("10% hypothetical tax total", sub + sub * F(10, 100), 2574)
client_claimed = 1650  # tampered hidden field
ok("server total differs from client claim", (sub, sub != client_claimed), (2340, True))

# --- CE 15.6 availability ---------------------------------------------------
serial = F(999, 1000) * F(999, 1000) * F(995, 1000)
print("    serial availability ~", float(serial))
ok("serial avail exact", serial, F(993010995, 10**9))
pair = 1 - (1 - F(995, 1000)) ** 2
ok("redundant DB pair", pair, F(999975, 10**6))
serial2 = F(999, 1000) * F(999, 1000) * pair
print("    with pair ~", float(serial2))
hours = 365 * 24
print("    downtime/yr single:", float((1 - serial) * hours), "h; with pair:", float((1 - serial2) * hours), "h")
# --- practice answers -------------------------------------------------------
ok("Q1 Amdahl", amdahl(F(8,10), 4), F(5, 2))
ok("Q1 limit", 1 / (1 - F(8,10)), F(5))
ok("Q1 Gustafson", F(2,10) + 4 * F(8,10), F(34, 10))
ok("Q2 concurrent with e", [x for x in "abcdfgh" if not hb(res[x][1], res['e'][1]) and not hb(res['e'][1], res[x][1])], list("abcd"))
q3 = len(xdr_int(7) + xdr_string("Pokhara"))
ok("Q3 args/msg/tcp", (q3, 40 + q3, 44 + q3), (16, 56, 60))
q6 = (15_000 // 30) * (12_000 // 30) * 7
ok("Q6 bytes 30 m", q6, 1_400_000)
ok("Q6 bytes 15 m", (15_000 // 15) * (12_000 // 15) * 7, 5_600_000)
ok("Q7 pixel", to_rc(501_234, 3_099_876), (12, 123))
ok("Q7 centre", centre(12, 123), (501_235, 3_099_875))
nb2 = [0, 52, 50, 49, 255, 51, 50, 48, 53]
ok("Q8 median", sorted(nb2)[4], 50)
ok("Q8 mean", F(sum(nb2), 9), F(608, 9)); print("    Q8 mean ~", sum(nb2) / 9)
ok("Q9 frames", (2 * 8000 - 160) // 80 + 1, 199)
ok("Q9 rate", 8000 * 8, 64_000)
q10 = (1 - F(1, 100) ** 2) * F(995, 1000)
ok("Q10 availability", q10, F(9949005, 10**7))
print("ALL CHECKS PASSED")
```
