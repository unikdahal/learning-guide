/**
 * subnet-math.ts
 *
 * Pure TypeScript IPv4 subnet mathematics library with zero runtime dependencies.
 * Provides strict parsing, unsigned 32-bit bitwise calculations, RFC 3021 (/31)
 * semantics, /32 single-host handling, /0 generic arithmetic vs routing notes,
 * step-by-step bitwise AND breakdowns, 32-bit binary visual strips, and bounded
 * equal-size subnet splitting.
 */

export interface IPv4Octets {
  readonly octets: readonly [number, number, number, number];
  readonly uint32: number;
}

export type ParseResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: string };

export type SubnetCategory = 'default-route' | 'standard' | 'point-to-point' | 'single-host';

export type AddressScope =
  | 'this-host'
  | 'private-10'
  | 'private-172'
  | 'private-192'
  | 'carrier-grade-nat'
  | 'loopback'
  | 'link-local'
  | 'documentation'
  | 'benchmark'
  | 'multicast'
  | 'reserved-e'
  | 'limited-broadcast'
  | 'ietf-protocol'
  | 'unclassified';

export interface BitStep {
  readonly octetIndex: 1 | 2 | 3 | 4;
  readonly ipDecimal: number;
  readonly ipBinary: string;
  readonly maskDecimal: number;
  readonly maskBinary: string;
  readonly networkDecimal: number;
  readonly networkBinary: string;
  readonly explanation: string;
}

export interface StripBit {
  readonly bitIndex: number; // 31 (MSB) down to 0 (LSB)
  readonly octetIndex: 0 | 1 | 2 | 3;
  readonly isNetwork: boolean;
  readonly ipBit: 0 | 1;
  readonly maskBit: 0 | 1;
  readonly networkBit: 0 | 1;
}

export interface SubnetCalculation {
  readonly rawIp: string;
  readonly ip: number;
  readonly ipStr: string;
  readonly ipOctets: readonly [number, number, number, number];
  readonly ipBinary: string;

  readonly prefix: number;
  readonly mask: number;
  readonly maskStr: string;
  readonly maskOctets: readonly [number, number, number, number];
  readonly maskBinary: string;

  readonly wildcard: number;
  readonly wildcardStr: string;

  readonly network: number;
  readonly networkStr: string;
  readonly networkBinary: string;

  readonly broadcast: number | null;
  readonly broadcastStr: string;

  readonly totalAddresses: number; // 2^(32-prefix)
  readonly usableHosts: number;

  readonly firstUsable: number | null;
  readonly firstUsableStr: string;
  readonly lastUsable: number | null;
  readonly lastUsableStr: string;
  readonly hostRangeStr: string;

  readonly category: SubnetCategory;
  readonly addressClass: string;
  readonly scope: AddressScope;
  readonly isPrivate: boolean;

  readonly interestingOctet: 1 | 2 | 3 | 4;
  readonly blockSize: number;

  readonly andSteps: readonly BitStep[];
  readonly stripBits: readonly StripBit[];
  readonly stripAccessibleText: string;
  readonly contextNote?: string;
}

export interface SubnetSplitRow {
  readonly index: number;
  readonly network: number;
  readonly networkStr: string;
  readonly cidrStr: string;
  readonly maskStr: string;
  readonly broadcastStr: string;
  readonly firstUsableStr: string;
  readonly lastUsableStr: string;
  readonly hostRangeStr: string;
  readonly usableHosts: number;
  readonly containsInputIp: boolean;
}

export interface SubnetSplitResult {
  readonly parentNetworkStr: string;
  readonly parentPrefix: number;
  readonly borrowedBits: number;
  readonly childPrefix: number;
  readonly totalSubnets: number;
  readonly displayedSubnets: number;
  readonly isTruncated: boolean;
  readonly hostsPerSubnet: number;
  readonly blockSize: number;
  readonly rows: readonly SubnetSplitRow[];
}

export interface WorkedExample {
  readonly id: string;
  readonly title: string;
  readonly syllabusTag: string; // e.g., 'NEC 5.3, CE 1.3'
  readonly prompt: string;
  readonly inputIp: string;
  readonly inputPrefix: number;
  readonly borrowedBits?: number;
  readonly solutionSummary: string;
  readonly rationale: string;
}

/**
 * Converts unsigned 32-bit number to dotted-quad string.
 */
export function uint32ToIPv4(val: number): string {
  const u = val >>> 0;
  return `${(u >>> 24) & 255}.${(u >>> 16) & 255}.${(u >>> 8) & 255}.${u & 255}`;
}

/**
 * Converts 4 octets to unsigned 32-bit number.
 */
export function octetsToUint32(octets: readonly [number, number, number, number]): number {
  return ((octets[0] * 16777216) + (octets[1] * 65536) + (octets[2] * 256) + octets[3]) >>> 0;
}

/**
 * Formats a number to an 8-bit binary string.
 */
export function byteToBinaryString(b: number): string {
  return (b & 255).toString(2).padStart(8, '0');
}

/**
 * Formats unsigned 32-bit number to 4 octets binary string separated by dot.
 */
export function uint32ToBinaryString(u: number, separator = '.'): string {
  const val = u >>> 0;
  return [
    byteToBinaryString((val >>> 24) & 255),
    byteToBinaryString((val >>> 16) & 255),
    byteToBinaryString((val >>> 8) & 255),
    byteToBinaryString(val & 255),
  ].join(separator);
}

/**
 * Strict IPv4 dotted-quad parser.
 * Rejects:
 * - Empty strings, whitespace inside octets, empty octets (no coerced 0s)
 * - Fewer or more than 4 octets
 * - Non-digit characters, leading zeros in multi-digit numbers (octal ambiguity)
 * - Values < 0 or > 255
 */
export function parseIPv4(input: string): ParseResult<IPv4Octets> {
  if (typeof input !== 'string') {
    return { ok: false, error: 'IPv4 input must be a string' };
  }

  const trimmed = input.trim();
  if (trimmed === '') {
    return { ok: false, error: 'Address cannot be empty' };
  }

  // Must not have trailing/leading dots or consecutive dots
  if (trimmed.startsWith('.') || trimmed.endsWith('.') || trimmed.includes('..')) {
    return { ok: false, error: 'Malformed dotted-quad format (consecutive or misplaced dots)' };
  }

  const parts = trimmed.split('.');
  if (parts.length !== 4) {
    return {
      ok: false,
      error: `IPv4 address must have exactly 4 octets separated by dots (received ${parts.length})`,
    };
  }

  const parsedOctets: [number, number, number, number] = [0, 0, 0, 0];

  for (let i = 0; i < 4; i++) {
    const part = parts[i];
    if (part === '') {
      return { ok: false, error: `Octet ${i + 1} is empty` };
    }
    // Only pure decimal digits allowed
    if (!/^\d+$/.test(part)) {
      return { ok: false, error: `Octet ${i + 1} contains non-decimal characters: "${part}"` };
    }
    // Disallow leading zeros for numbers > 0 (e.g. "01" or "09")
    if (part.length > 1 && part.startsWith('0')) {
      return {
        ok: false,
        error: `Octet ${i + 1} has forbidden leading zero "${part}" (octal ambiguity)`,
      };
    }

    const num = Number(part);
    if (!Number.isSafeInteger(num) || num < 0 || num > 255) {
      return { ok: false, error: `Octet ${i + 1} value ${num} is out of range (must be 0–255)` };
    }

    parsedOctets[i] = num;
  }

  const uint32 = octetsToUint32(parsedOctets);
  return {
    ok: true,
    value: {
      octets: parsedOctets,
      uint32,
    },
  };
}

/**
 * Strict prefix validator (0..32).
 */
export function parsePrefix(input: string | number): ParseResult<number> {
  if (input === null || input === undefined) {
    return { ok: false, error: 'Prefix cannot be empty' };
  }

  let num: number;
  if (typeof input === 'string') {
    const trimmed = input.trim();
    if (trimmed === '') {
      return { ok: false, error: 'Prefix cannot be empty' };
    }
    if (!/^\d+$/.test(trimmed)) {
      return { ok: false, error: `Prefix "${input}" must contain only digits` };
    }
    num = Number(trimmed);
  } else {
    num = input;
  }

  if (!Number.isFinite(num) || !Number.isInteger(num)) {
    return { ok: false, error: `Prefix "${input}" must be an integer` };
  }

  if (num < 0 || num > 32) {
    return { ok: false, error: `Prefix length /${num} is out of range (must be 0–32)` };
  }

  return { ok: true, value: num };
}

/**
 * Generates 32-bit unsigned subnet mask from prefix length 0..32.
 * Handled with mathematical powers of 2 to eliminate 32-bit JS shift modulo quirks.
 */
export function prefixToMask(prefix: number): number {
  if (prefix < 0 || prefix > 32) {
    throw new RangeError(`Prefix ${prefix} out of bounds (0–32)`);
  }
  if (prefix === 0) return 0;
  if (prefix === 32) return 0xffffffff;
  // 2^32 - 2^(32-prefix) is exact in double precision (up to 2^53)
  return (2 ** 32 - 2 ** (32 - prefix)) >>> 0;
}

/**
 * Converts a 32-bit mask to prefix length, or null if non-contiguous.
 */
export function maskToPrefix(mask: number): number | null {
  const u = mask >>> 0;
  for (let p = 0; p <= 32; p++) {
    if (prefixToMask(p) === u) return p;
  }
  return null;
}

/**
 * Determines address class (historical classful model).
 */
export function getAddressClass(ip: number): string {
  const firstOctet = (ip >>> 24) & 255;
  if (firstOctet === 0) return '0.0.0.0/8 (This network)';
  if (firstOctet === 127) return 'Loopback (127.0.0.0/8)';
  if (firstOctet < 128) return 'Class A (1.0.0.0 – 126.255.255.255)';
  if (firstOctet < 192) return 'Class B (128.0.0.0 – 191.255.255.255)';
  if (firstOctet < 224) return 'Class C (192.0.0.0 – 223.255.255.255)';
  if (firstOctet < 240) return 'Class D (224.0.0.0 – 239.255.255.255 Multicast)';
  return 'Class E (240.0.0.0 – 255.255.255.255 Reserved)';
}

/**
 * Determines address scope according to RFC 1918, RFC 3927, RFC 6598, etc.
 */
export function getAddressScope(ip: number): { scope: AddressScope; isPrivate: boolean } {
  const u = ip >>> 0;
  const o0 = (u >>> 24) & 255;
  const o1 = (u >>> 16) & 255;
  const o2 = (u >>> 8) & 255;

  if (o0 === 0) return { scope: 'this-host', isPrivate: false };
  if (u === 0xffffffff) return { scope: 'limited-broadcast', isPrivate: false };
  if (o0 === 127) return { scope: 'loopback', isPrivate: false };
  if (o0 === 10) return { scope: 'private-10', isPrivate: true };
  if (o0 === 172 && o1 >= 16 && o1 <= 31) return { scope: 'private-172', isPrivate: true };
  if (o0 === 192 && o1 === 168) return { scope: 'private-192', isPrivate: true };
  if (o0 === 100 && o1 >= 64 && o1 <= 127) return { scope: 'carrier-grade-nat', isPrivate: false };
  if (o0 === 169 && o1 === 254) return { scope: 'link-local', isPrivate: false };
  if (o0 === 192 && o1 === 0 && o2 === 0) return { scope: 'ietf-protocol', isPrivate: false };
  if (o0 === 192 && o1 === 0 && o2 === 2) return { scope: 'documentation', isPrivate: false };
  if (o0 === 198 && o1 === 51 && o2 === 100) return { scope: 'documentation', isPrivate: false };
  if (o0 === 203 && o1 === 0 && o2 === 113) return { scope: 'documentation', isPrivate: false };
  if (o0 === 198 && (o1 === 18 || o1 === 19)) return { scope: 'benchmark', isPrivate: false };
  if (o0 >= 224 && o0 <= 239) return { scope: 'multicast', isPrivate: false };
  if (o0 >= 240) return { scope: 'reserved-e', isPrivate: false };

  return { scope: 'unclassified', isPrivate: false };
}

/**
 * Computes step-by-step AND calculations for each octet.
 */
export function computeAndSteps(
  ipOctets: readonly [number, number, number, number],
  maskOctets: readonly [number, number, number, number],
): BitStep[] {
  const steps: BitStep[] = [];
  for (let i = 0; i < 4; i++) {
    const ipDec = ipOctets[i];
    const maskDec = maskOctets[i];
    const netDec = ipDec & maskDec;
    const ipBin = byteToBinaryString(ipDec);
    const maskBin = byteToBinaryString(maskDec);
    const netBin = byteToBinaryString(netDec);
    steps.push({
      octetIndex: (i + 1) as 1 | 2 | 3 | 4,
      ipDecimal: ipDec,
      ipBinary: ipBin,
      maskDecimal: maskDec,
      maskBinary: maskBin,
      networkDecimal: netDec,
      networkBinary: netBin,
      explanation: `Octet ${i + 1}: ${ipDec} AND ${maskDec} = ${netDec} (${ipBin} AND ${maskBin} = ${netBin})`,
    });
  }
  return steps;
}

/**
 * Generates the 32-bit strip breakdown and accessible text.
 */
export function computeStripBits(ip: number, mask: number, prefix: number): {
  stripBits: StripBit[];
  accessibleText: string;
} {
  const uIp = ip >>> 0;
  const uMask = mask >>> 0;
  const uNet = (uIp & uMask) >>> 0;

  const bits: StripBit[] = [];

  for (let i = 31; i >= 0; i--) {
    const bitPosFromLeft = 31 - i; // 0..31
    const octetIdx = Math.floor(bitPosFromLeft / 8) as 0 | 1 | 2 | 3;
    const isNet = bitPosFromLeft < prefix;

    bits.push({
      bitIndex: i,
      octetIndex: octetIdx,
      isNetwork: isNet,
      ipBit: ((uIp >>> i) & 1) as 0 | 1,
      maskBit: ((uMask >>> i) & 1) as 0 | 1,
      networkBit: ((uNet >>> i) & 1) as 0 | 1,
    });
  }

  const ipStr = uint32ToIPv4(uIp);
  const netStr = uint32ToIPv4(uNet);
  const maskStr = uint32ToIPv4(uMask);

  const accessibleText =
    `32-bit IPv4 structure for ${ipStr}/${prefix}: Network bits 1–${prefix} (mask 1s), Host bits ${
      prefix + 1
    }–32 (mask 0s). ` +
    `Binary IP: ${uint32ToBinaryString(uIp)}. Mask: ${maskStr} (${uint32ToBinaryString(uMask)}). Network: ${
      uint32ToBinaryString(uNet)
    }. Network Address: ${netStr}.`;

  return { stripBits: bits, accessibleText };
}

/**
 * Calculates complete subnet parameters for given IP and prefix.
 *
 * Implements:
 * - /32: 1 host, no separate network or broadcast (single host route / loopback).
 * - /31: RFC 3021 point-to-point link. Exactly 2 usable hosts (both assigned to router interfaces),
 *        no directed broadcast packet permitted.
 * - /0: 4,294,967,296 addresses; generic arithmetic 4,294,967,294 usable addresses,
 *       with explicit route-context documentation note (default route 0.0.0.0/0).
 * - /1../30: Conventional LAN with network address (host bits 0) and broadcast (host bits 1).
 */
export function calculateSubnet(
  ipInput: string | number,
  prefixInput: string | number,
): SubnetCalculation {
  let ip: number;
  let rawIp: string;

  if (typeof ipInput === 'number') {
    if (!Number.isSafeInteger(ipInput) || ipInput < 0 || ipInput > 0xffffffff) {
      throw new RangeError(`Invalid IP integer ${ipInput}`);
    }
    ip = ipInput >>> 0;
    rawIp = uint32ToIPv4(ip);
  } else {
    rawIp = String(ipInput);
    const parseRes = parseIPv4(rawIp);
    if (!parseRes.ok) {
      throw new Error(parseRes.error);
    }
    ip = parseRes.value.uint32;
  }

  const prefRes = parsePrefix(prefixInput);
  if (!prefRes.ok) {
    throw new Error(prefRes.error);
  }
  const prefix = prefRes.value;

  const mask = prefixToMask(prefix);
  const wildcard = (~mask) >>> 0;
  const network = (ip & mask) >>> 0;

  const ipStr = uint32ToIPv4(ip);
  const maskStr = uint32ToIPv4(mask);
  const wildcardStr = uint32ToIPv4(wildcard);
  const networkStr = uint32ToIPv4(network);

  const ipOctets: [number, number, number, number] = [
    (ip >>> 24) & 255,
    (ip >>> 16) & 255,
    (ip >>> 8) & 255,
    ip & 255,
  ];

  const maskOctets: [number, number, number, number] = [
    (mask >>> 24) & 255,
    (mask >>> 16) & 255,
    (mask >>> 8) & 255,
    mask & 255,
  ];

  const ipBinary = uint32ToBinaryString(ip);
  const maskBinary = uint32ToBinaryString(mask);
  const networkBinary = uint32ToBinaryString(network);

  const totalAddresses = 2 ** (32 - prefix);

  // Interesting octet calculation (which octet holds the boundary)
  let interestingOctet: 1 | 2 | 3 | 4 = 4;
  if (prefix <= 8) interestingOctet = 1;
  else if (prefix <= 16) interestingOctet = 2;
  else if (prefix <= 24) interestingOctet = 3;
  else interestingOctet = 4;

  const bitsInInterestingOctet = prefix % 8 === 0 ? (prefix === 0 ? 0 : 8) : prefix % 8;
  const blockSize = 2 ** (8 - bitsInInterestingOctet);

  let category: SubnetCategory;
  let broadcast: number | null = null;
  let broadcastStr = '';
  let usableHosts = 0;
  let firstUsable: number | null = null;
  let firstUsableStr = '';
  let lastUsable: number | null = null;
  let lastUsableStr = '';
  let hostRangeStr = '';
  let contextNote: string | undefined;

  if (prefix === 32) {
    category = 'single-host';
    broadcast = null;
    broadcastStr = 'None (/32 single host)';
    usableHosts = 1;
    firstUsable = network;
    firstUsableStr = networkStr;
    lastUsable = network;
    lastUsableStr = networkStr;
    hostRangeStr = `${networkStr} (single host / loopback)`;
    contextNote =
      'A /32 prefix designates a single IPv4 host or loopback route. It contains no dedicated network or broadcast addresses.';
  } else if (prefix === 31) {
    category = 'point-to-point';
    // RFC 3021
    broadcast = null;
    broadcastStr = 'None (RFC 3021 point-to-point link)';
    usableHosts = 2;
    firstUsable = network;
    firstUsableStr = uint32ToIPv4(network);
    lastUsable = (network + 1) >>> 0;
    lastUsableStr = uint32ToIPv4(lastUsable);
    hostRangeStr = `${firstUsableStr} – ${lastUsableStr} (RFC 3021 endpoints)`;
    contextNote =
      'RFC 3021 allows 31-bit prefixes on point-to-point links. Both addresses (host bits 0 and 1) are assigned to the two router interfaces. Directed broadcast is prohibited on /31 links.';
  } else if (prefix === 0) {
    category = 'default-route';
    broadcast = 0xffffffff;
    broadcastStr = '255.255.255.255';
    // In generic 32-bit arithmetic: 2^32 - 2
    usableHosts = 4294967294;
    firstUsable = 1;
    firstUsableStr = '0.0.0.1';
    lastUsable = 0xfffffffe;
    lastUsableStr = '255.255.255.254';
    hostRangeStr = '0.0.0.1 – 255.255.255.254 (generic arithmetic)';
    contextNote =
      '0.0.0.0/0 represents the default route (the entire IPv4 internet). The 4,294,967,294 host count reflects generic 32-bit arithmetic, not a physical host LAN configuration.';
  } else {
    category = 'standard';
    broadcast = (network | wildcard) >>> 0;
    broadcastStr = uint32ToIPv4(broadcast);
    usableHosts = totalAddresses - 2;
    firstUsable = (network + 1) >>> 0;
    firstUsableStr = uint32ToIPv4(firstUsable);
    lastUsable = (broadcast - 1) >>> 0;
    lastUsableStr = uint32ToIPv4(lastUsable);
    hostRangeStr = `${firstUsableStr} – ${lastUsableStr}`;
  }

  const addressClass = getAddressClass(ip);
  const { scope, isPrivate } = getAddressScope(ip);
  const andSteps = computeAndSteps(ipOctets, maskOctets);
  const { stripBits, accessibleText } = computeStripBits(ip, mask, prefix);

  return {
    rawIp,
    ip,
    ipStr,
    ipOctets,
    ipBinary,

    prefix,
    mask,
    maskStr,
    maskOctets,
    maskBinary,

    wildcard,
    wildcardStr,

    network,
    networkStr,
    networkBinary,

    broadcast,
    broadcastStr,

    totalAddresses,
    usableHosts,

    firstUsable,
    firstUsableStr,
    lastUsable,
    lastUsableStr,
    hostRangeStr,

    category,
    addressClass,
    scope,
    isPrivate,

    interestingOctet,
    blockSize,

    andSteps,
    stripBits,
    stripAccessibleText: accessibleText,
    contextNote,
  };
}

/**
 * Splits a parent CIDR into equal-sized child subnets by borrowing bits.
 * Bounded table output avoids rendering millions of rows into DOM or memory.
 */
export function splitSubnet(
  parentIp: string | number,
  parentPrefix: number,
  borrowedBits: number,
  maxDisplay = 16,
): SubnetSplitResult {
  const prefRes = parsePrefix(parentPrefix);
  if (!prefRes.ok) throw new Error(prefRes.error);
  const p = prefRes.value;

  if (
    !Number.isInteger(borrowedBits) ||
    borrowedBits < 1 ||
    borrowedBits > 32 - p
  ) {
    throw new RangeError(
      `Borrowed bits must be an integer between 1 and ${32 - p} (received ${borrowedBits})`,
    );
  }

  const childPrefix = p + borrowedBits;
  const totalSubnets = 2 ** borrowedBits;
  const parentCalc = calculateSubnet(parentIp, p);
  const parentNet = parentCalc.network;
  const childSubnetSize = 2 ** (32 - childPrefix);

  const displayLimit = Math.min(totalSubnets, Math.max(1, maxDisplay));
  const rows: SubnetSplitRow[] = [];

  const childMask = prefixToMask(childPrefix);
  const childMaskStr = uint32ToIPv4(childMask);

  let hostsPerSubnet = 0;
  if (childPrefix === 32) hostsPerSubnet = 1;
  else if (childPrefix === 31) hostsPerSubnet = 2;
  else hostsPerSubnet = childSubnetSize - 2;

  for (let i = 0; i < displayLimit; i++) {
    const subNet = (parentNet + i * childSubnetSize) >>> 0;
    const subCalc = calculateSubnet(subNet, childPrefix);
    const containsInput = parentCalc.ip >= subNet && parentCalc.ip < subNet + childSubnetSize;

    rows.push({
      index: i,
      network: subNet,
      networkStr: subCalc.networkStr,
      cidrStr: `${subCalc.networkStr}/${childPrefix}`,
      maskStr: childMaskStr,
      broadcastStr: subCalc.broadcastStr,
      firstUsableStr: subCalc.firstUsableStr,
      lastUsableStr: subCalc.lastUsableStr,
      hostRangeStr: subCalc.hostRangeStr,
      usableHosts: subCalc.usableHosts,
      containsInputIp: containsInput,
    });
  }

  // Calculate block size in interesting octet of child subnet
  const bitsInChildInterestingOctet =
    childPrefix % 8 === 0 ? (childPrefix === 0 ? 0 : 8) : childPrefix % 8;
  const childBlockSize = 2 ** (8 - bitsInChildInterestingOctet);

  return {
    parentNetworkStr: parentCalc.networkStr,
    parentPrefix: p,
    borrowedBits,
    childPrefix,
    totalSubnets,
    displayedSubnets: displayLimit,
    isTruncated: totalSubnets > displayLimit,
    hostsPerSubnet,
    blockSize: childBlockSize,
    rows,
  };
}

/**
 * 6+ original fixed worked examples with full answers and explanations.
 * Static server-side fallback and interactive lab presets share this exact dataset.
 */
export const WORKED_EXAMPLES: readonly WorkedExample[] = [
  {
    id: 'lok-sewa-200-168-10-75',
    title: 'Lok Sewa CE 1.3 / NEC 5.3: Subnetting 200.168.10.0/24 for 7 subnets (Host .75)',
    syllabusTag: 'NEC 5.3 · CE 1.3 CO 7',
    prompt:
      'An organisation is assigned the Class C network 200.168.10.0/24 and requires 7 subnets. Find the new subnet mask, hosts per subnet, block size, all subnet addresses, and the specific network, broadcast, and valid host range for host 200.168.10.75.',
    inputIp: '200.168.10.75',
    inputPrefix: 24,
    borrowedBits: 3,
    solutionSummary:
      'Borrow s = ⌈log₂ 7⌉ = 3 bits → New prefix /27 (255.255.255.224). Block size = 32. Usable hosts = 30 per subnet. Host 200.168.10.75 falls in subnet 2 (200.168.10.64/27), broadcast 200.168.10.95, usable range 200.168.10.65 – 200.168.10.94.',
    rationale:
      'Applying bitwise AND between 200.168.10.75 (octet 4: 75 = 01001011₂) and mask 255.255.255.224 (octet 4: 224 = 11100000₂) yields 64 (01000000₂). Subnets increment by 32 in the fourth octet: .0, .32, .64, .96, .128, .160, .192, .224. 75 lies between 64 and 95.',
  },
  {
    id: 'lok-sewa-ce-2078-host-177',
    title: 'Reported CE 2078 transcription (unverified): Host .177 on /26',
    syllabusTag: 'NEC 5.3 · CE 1.3 · Unverified historical transcription',
    prompt:
      'For host IP 192.168.1.177 with subnet mask 255.255.255.192 (/26), determine the network address, broadcast address, first usable host, last usable host, and total number of usable host addresses.',
    inputIp: '192.168.1.177',
    inputPrefix: 26,
    borrowedBits: 2,
    solutionSummary:
      'Network: 192.168.1.128/26. Broadcast: 192.168.1.191. Usable host range: 192.168.1.129 – 192.168.1.190. Usable hosts: 62 (2⁶ - 2).',
    rationale:
      'Mask /26 has 6 host bits. Block size in the 4th octet is 256 - 192 = 64. Subnets are .0, .64, .128, .192. Host 177 falls in 128..191. AND verification: 177 (10110001₂) AND 192 (11000000₂) = 128 (10000000₂).',
  },
  {
    id: 'rfc-3021-point-to-point',
    title: 'RFC 3021 Point-to-Point Link: 10.0.0.1/31 Router Interconnect',
    syllabusTag: 'RFC 3021 · Core Routing',
    prompt:
      'Explain the address structure and host usability of 10.0.0.1/31 on a high-speed point-to-point router link according to RFC 3021.',
    inputIp: '10.0.0.1',
    inputPrefix: 31,
    solutionSummary:
      'Network: 10.0.0.0/31. Directed broadcast: None. Exactly 2 usable hosts: 10.0.0.0 and 10.0.0.1, assigned to both router endpoints.',
    rationale:
      'On classical multi-access LANs, /31 would leave 2 addresses with 0 usable hosts because host bits all-0 (network) and all-1 (broadcast) are reserved. RFC 3021 explicitly eliminates directed broadcasts on point-to-point links, conserving IPv4 space by allowing host bit 0 and host bit 1 to be assigned to the two router interfaces.',
  },
  {
    id: 'single-host-32-loopback',
    title: 'Single Host /32 Loopback & Host Route: 172.16.100.5/32',
    syllabusTag: 'NEC 5.3 · Routing Architecture',
    prompt:
      'Determine the subnet attributes of router loopback address 172.16.100.5 configured with a /32 prefix.',
    inputIp: '172.16.100.5',
    inputPrefix: 32,
    solutionSummary:
      'Subnet mask: 255.255.255.255. Network: 172.16.100.5. Broadcast: None. Usable host: exactly 1 (172.16.100.5).',
    rationale:
      'A /32 prefix leaves 0 host bits (2⁰ = 1 address). It identifies an individual endpoint, management interface, or host route in routing tables without any broadcast domain overhead.',
  },
  {
    id: 'class-b-cidr-split',
    title: 'Enterprise Class B Subnetting: 172.16.0.0/16 split by 4 bits into /20',
    syllabusTag: 'NEC 5.3 · VLSM & CIDR',
    prompt:
      'A campus network uses private address space 172.16.0.0/16 and borrows 4 bits to create department subnets. Find the new prefix, mask, block size in the interesting octet, total subnets, and host capacity per subnet.',
    inputIp: '172.16.45.10',
    inputPrefix: 16,
    borrowedBits: 4,
    solutionSummary:
      'Borrowing 4 bits yields prefix /20 (mask 255.255.240.0). Creates 16 subnets. Interesting octet is the 3rd octet with block size 256 - 240 = 16. Each subnet supports 4,094 usable hosts (2¹² - 2). Host 172.16.45.10 is in subnet 172.16.32.0/20 (range 172.16.32.1 – 172.16.47.254, broadcast 172.16.47.255).',
    rationale:
      'Third octet value 45 divided by block size 16 has quotient 2 (32). Subnet boundary is 172.16.32.0; next subnet is 172.16.48.0. Broadcast is 172.16.47.255. Host .45.10 sits comfortably inside.',
  },
  {
    id: 'default-route-boundary-0',
    title: 'Default Route Boundary /0: 0.0.0.0/0 Generic Arithmetic vs Routing Context',
    syllabusTag: 'NEC 5.3 · Internet Architecture',
    prompt:
      'Analyse the boundary prefix 0.0.0.0/0. Explain its mathematical output versus its practical application in internet routing tables.',
    inputIp: '0.0.0.0',
    inputPrefix: 0,
    solutionSummary:
      'Mask: 0.0.0.0. Total addresses: 4,294,967,296 (entire IPv4 space). Generic arithmetic yields 4,294,967,294 usable addresses. In network routing, 0.0.0.0/0 is the default route (gateway of last resort), not a physical LAN broadcast domain.',
    rationale:
      'A prefix of 0 network bits means all 32 bits are host bits. Bitwise AND with mask 0.0.0.0 yields network 0.0.0.0. In routing protocols (BGP, OSPF) and system routing tables, 0.0.0.0/0 matches any destination IP when no more-specific route exists.',
  },
  {
    id: 'unsigned-high-boundary',
    title: 'High Unsigned Boundary Test: 223.255.255.254/29',
    syllabusTag: 'Core Arithmetic · Unsigned Boundary',
    prompt:
      'Verify that 32-bit arithmetic handles high octet values (MSB set, octets ≥ 128) without 32-bit signed two’s-complement corruption for IP 223.255.255.254/29.',
    inputIp: '223.255.255.254',
    inputPrefix: 29,
    borrowedBits: 1,
    solutionSummary:
      'Mask: 255.255.255.248 (/29). Network: 223.255.255.248. Broadcast: 223.255.255.255. Usable host range: 223.255.255.249 – 223.255.255.254. Usable hosts: 6.',
    rationale:
      'Octet 223 has MSB 1 (11011111₂). In JavaScript, bitwise operations interpret bit 31 as negative unless zero-fill right-shifted (>>> 0). 223.255.255.254 represents unsigned integer 3758096382, which evaluates correctly without signed bit overflow.',
  },
];
