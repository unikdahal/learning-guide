import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseIPv4,
  parsePrefix,
  prefixToMask,
  maskToPrefix,
  uint32ToIPv4,
  octetsToUint32,
  calculateSubnet,
  splitSubnet,
  getAddressClass,
  getAddressScope,
  WORKED_EXAMPLES,
} from '../../src/lib/subnet-math.ts';

test('strict IPv4 parser rejects malformed inputs and empty octet coercion', () => {
  const malformed = [
    '',
    '   ',
    '192.168.1',
    '192.168.1.1.1',
    '192.168..1',
    '192.168.1.',
    '.192.168.1.1',
    '192.168.01.1', // leading zero
    '192.168.00.1',
    '192.168.1.256',
    '192.168.1.-1',
    '192.168.1.1a',
    '192. 168.1.1',
    '192.168.1. 1',
    '0x12.0x34.0x56.0x78',
    'hello',
    '256.255.255.255',
    '1.2.3.999',
  ];

  for (const bad of malformed) {
    const res = parseIPv4(bad);
    assert.equal(res.ok, false, `Expected parseIPv4 to reject "${bad}"`);
  }

  // Valid inputs
  const valid = [
    { in: '0.0.0.0', octets: [0, 0, 0, 0], uint32: 0 },
    { in: '255.255.255.255', octets: [255, 255, 255, 255], uint32: 4294967295 },
    { in: '192.168.1.1', octets: [192, 168, 1, 1], uint32: 3232235777 },
    { in: '10.0.0.1', octets: [10, 0, 0, 1], uint32: 167772161 },
    { in: '128.0.0.1', octets: [128, 0, 0, 1], uint32: 2147483649 },
  ];

  for (const v of valid) {
    const res = parseIPv4(v.in);
    assert.equal(res.ok, true, `Expected valid for "${v.in}"`);
    if (res.ok) {
      assert.deepEqual(res.value.octets, v.octets);
      assert.equal(res.value.uint32, v.uint32);
    }
  }
});

test('prefix parser validates strict range 0..32 and rejects non-integers', () => {
  for (let p = 0; p <= 32; p++) {
    const resNum = parsePrefix(p);
    assert.equal(resNum.ok, true);
    if (resNum.ok) assert.equal(resNum.value, p);

    const resStr = parsePrefix(String(p));
    assert.equal(resStr.ok, true);
    if (resStr.ok) assert.equal(resStr.value, p);
  }

  const badPrefixes = [-1, 33, 100, -100, 24.5, '24.5', 'abc', '', ' ', NaN, Infinity, -Infinity];
  for (const bad of badPrefixes) {
    const res = parsePrefix(bad as any);
    assert.equal(res.ok, false, `Expected bad prefix for "${bad}"`);
  }
});

test('prefixToMask handles boundary 0, 31, 32 without signed 32-bit shift errors', () => {
  assert.equal(prefixToMask(0), 0);
  assert.equal(uint32ToIPv4(prefixToMask(0)), '0.0.0.0');

  assert.equal(prefixToMask(1), 2147483648);
  assert.equal(uint32ToIPv4(prefixToMask(1)), '128.0.0.0');

  assert.equal(prefixToMask(8), 4278190080);
  assert.equal(uint32ToIPv4(prefixToMask(8)), '255.0.0.0');

  assert.equal(prefixToMask(16), 4294901760);
  assert.equal(uint32ToIPv4(prefixToMask(16)), '255.255.0.0');

  assert.equal(prefixToMask(24), 4294967040);
  assert.equal(uint32ToIPv4(prefixToMask(24)), '255.255.255.0');

  assert.equal(prefixToMask(30), 4294967292);
  assert.equal(uint32ToIPv4(prefixToMask(30)), '255.255.255.252');

  assert.equal(prefixToMask(31), 4294967294);
  assert.equal(uint32ToIPv4(prefixToMask(31)), '255.255.255.254');

  assert.equal(prefixToMask(32), 4294967295);
  assert.equal(uint32ToIPv4(prefixToMask(32)), '255.255.255.255');

  // Round trip with maskToPrefix
  for (let p = 0; p <= 32; p++) {
    const mask = prefixToMask(p);
    assert.equal(maskToPrefix(mask), p, `Round trip failed for prefix ${p}`);
  }
  assert.equal(maskToPrefix(0x12345678), null);
});

test('boundary /31 point-to-point implements RFC 3021 semantics', () => {
  const calc = calculateSubnet('10.0.0.1', 31);
  assert.equal(calc.category, 'point-to-point');
  assert.equal(calc.networkStr, '10.0.0.0');
  assert.equal(calc.broadcast, null);
  assert.match(calc.broadcastStr, /RFC 3021/);
  assert.equal(calc.usableHosts, 2);
  assert.equal(calc.firstUsableStr, '10.0.0.0');
  assert.equal(calc.lastUsableStr, '10.0.0.1');
  assert.match(calc.hostRangeStr, /RFC 3021 endpoints/);
  assert.ok(calc.contextNote?.includes('RFC 3021'));
});

test('boundary /32 single host implements host-route semantics', () => {
  const calc = calculateSubnet('172.16.100.5', 32);
  assert.equal(calc.category, 'single-host');
  assert.equal(calc.networkStr, '172.16.100.5');
  assert.equal(calc.maskStr, '255.255.255.255');
  assert.equal(calc.broadcast, null);
  assert.match(calc.broadcastStr, /\/32 single host/);
  assert.equal(calc.usableHosts, 1);
  assert.equal(calc.firstUsableStr, '172.16.100.5');
  assert.equal(calc.lastUsableStr, '172.16.100.5');
  assert.match(calc.hostRangeStr, /single host/);
});

test('boundary /0 handles 4,294,967,296 space with generic arithmetic and routing context note', () => {
  const calc = calculateSubnet('0.0.0.0', 0);
  assert.equal(calc.category, 'default-route');
  assert.equal(calc.networkStr, '0.0.0.0');
  assert.equal(calc.maskStr, '0.0.0.0');
  assert.equal(calc.broadcastStr, '255.255.255.255');
  assert.equal(calc.totalAddresses, 4294967296);
  assert.equal(calc.usableHosts, 4294967294);
  assert.equal(calc.firstUsableStr, '0.0.0.1');
  assert.equal(calc.lastUsableStr, '255.255.254.254' in calc ? '' : '255.255.255.254');
  assert.ok(calc.contextNote?.includes('default route'));
});

test('unsigned >= 128 calculations maintain mathematical invariants without negative numbers', () => {
  const testAddresses = [
    { ip: '128.0.0.1', prefix: 16, net: '128.0.0.0', bc: '128.0.255.255' },
    { ip: '192.168.254.254', prefix: 24, net: '192.168.254.0', bc: '192.168.254.255' },
    { ip: '200.168.10.75', prefix: 27, net: '200.168.10.64', bc: '200.168.10.95' },
    { ip: '223.255.255.254', prefix: 29, net: '223.255.255.248', bc: '223.255.255.255' },
    { ip: '240.10.20.30', prefix: 12, net: '240.0.0.0', bc: '240.15.255.255' },
  ];

  for (const item of testAddresses) {
    const calc = calculateSubnet(item.ip, item.prefix);
    assert.ok(calc.ip >= 0 && calc.ip <= 4294967295, 'IP uint32 must be non-negative');
    assert.ok(calc.mask >= 0 && calc.mask <= 4294967295, 'Mask uint32 must be non-negative');
    assert.ok(calc.network >= 0 && calc.network <= 4294967295, 'Network uint32 must be non-negative');
    assert.equal(calc.networkStr, item.net);
    assert.equal(calc.broadcastStr, item.bc);

    // Invariants
    assert.equal((calc.network & ~calc.mask) >>> 0, 0, 'No host bits in network address');
    if (calc.broadcast !== null) {
      assert.equal((calc.broadcast & ~calc.mask) >>> 0, calc.wildcard, 'All host bits in broadcast');
      assert.ok(calc.firstUsable! <= calc.lastUsable!, 'firstUsable <= lastUsable');
      assert.equal(
        calc.lastUsable! - calc.firstUsable! + 1,
        calc.usableHosts,
        'Usable host range span matches usableHosts count',
      );
    }
  }
});

test('step-by-step bitwise AND calculation produces correct arithmetic for 4 octets', () => {
  const calc = calculateSubnet('200.168.10.75', 27);
  assert.equal(calc.andSteps.length, 4);

  // Octet 1: 200 AND 255 = 200
  assert.equal(calc.andSteps[0].ipDecimal, 200);
  assert.equal(calc.andSteps[0].maskDecimal, 255);
  assert.equal(calc.andSteps[0].networkDecimal, 200);

  // Octet 2: 168 AND 255 = 168
  assert.equal(calc.andSteps[1].ipDecimal, 168);
  assert.equal(calc.andSteps[1].maskDecimal, 255);
  assert.equal(calc.andSteps[1].networkDecimal, 168);

  // Octet 3: 10 AND 255 = 10
  assert.equal(calc.andSteps[2].ipDecimal, 10);
  assert.equal(calc.andSteps[2].maskDecimal, 255);
  assert.equal(calc.andSteps[2].networkDecimal, 10);

  // Octet 4: 75 AND 224 = 64
  assert.equal(calc.andSteps[3].ipDecimal, 75);
  assert.equal(calc.andSteps[3].ipBinary, '01001011');
  assert.equal(calc.andSteps[3].maskDecimal, 224);
  assert.equal(calc.andSteps[3].maskBinary, '11100000');
  assert.equal(calc.andSteps[3].networkDecimal, 64);
  assert.equal(calc.andSteps[3].networkBinary, '01000000');
});

test('32-bit visual strip contains exactly 32 bits with correct network/host tags and accessible text', () => {
  const calc = calculateSubnet('192.168.1.10', 24);
  assert.equal(calc.stripBits.length, 32);

  // First 24 bits are network bits
  for (let i = 0; i < 24; i++) {
    assert.equal(calc.stripBits[i].isNetwork, true, `Bit ${i} should be network bit`);
  }
  // Remaining 8 bits are host bits
  for (let i = 24; i < 32; i++) {
    assert.equal(calc.stripBits[i].isNetwork, false, `Bit ${i} should be host bit`);
  }

  assert.ok(calc.stripAccessibleText.includes('Network bits 1–24'));
  assert.ok(calc.stripAccessibleText.includes('Host bits 25–32'));
  assert.ok(calc.stripAccessibleText.includes('192.168.1.10/24'));
});

test('address class and RFC scope classification accurately identify network categories', () => {
  // Test octetsToUint32
  assert.equal(octetsToUint32([10, 0, 0, 1]), 167772161);
  assert.equal(octetsToUint32([192, 168, 1, 1]), 3232235777);

  // Class tests
  assert.match(getAddressClass(0x0a000001), /Class A/);
  assert.match(getAddressClass(0x80000001), /Class B/);
  assert.match(getAddressClass(0xc0a80101), /Class C/);
  assert.match(getAddressClass(0xe0000001), /Class D/);
  assert.match(getAddressClass(0xf0000001), /Class E/);
  assert.match(getAddressClass(0x7f000001), /Loopback/);

  // Scope tests
  assert.equal(getAddressScope(0x0a000001).scope, 'private-10');
  assert.equal(getAddressScope(0x0a000001).isPrivate, true);

  const ip172_16 = parseIPv4('172.16.5.1');
  assert.ok(ip172_16.ok);
  if (ip172_16.ok) {
    assert.equal(getAddressScope(ip172_16.value.uint32).scope, 'private-172');
    assert.equal(getAddressScope(ip172_16.value.uint32).isPrivate, true);
  }

  const ip192_168 = parseIPv4('192.168.1.1');
  assert.ok(ip192_168.ok);
  if (ip192_168.ok) {
    assert.equal(getAddressScope(ip192_168.value.uint32).scope, 'private-192');
    assert.equal(getAddressScope(ip192_168.value.uint32).isPrivate, true);
  }

  const ipCgn = parseIPv4('100.64.0.1');
  assert.ok(ipCgn.ok);
  if (ipCgn.ok) {
    assert.equal(getAddressScope(ipCgn.value.uint32).scope, 'carrier-grade-nat');
  }

  const ipLinkLocal = parseIPv4('169.254.1.1');
  assert.ok(ipLinkLocal.ok);
  if (ipLinkLocal.ok) {
    assert.equal(getAddressScope(ipLinkLocal.value.uint32).scope, 'link-local');
  }
});

test('equal-size subnet splitting produces bounded table and correctly identifies host subnet', () => {
  // Split 200.168.10.75/24 with 3 borrowed bits (8 subnets of /27)
  const split = splitSubnet('200.168.10.75', 24, 3, 16);
  assert.equal(split.childPrefix, 27);
  assert.equal(split.totalSubnets, 8);
  assert.equal(split.displayedSubnets, 8);
  assert.equal(split.isTruncated, false);
  assert.equal(split.hostsPerSubnet, 30);
  assert.equal(split.blockSize, 32);

  // Subnet 0: 200.168.10.0/27
  assert.equal(split.rows[0].networkStr, '200.168.10.0');
  assert.equal(split.rows[0].containsInputIp, false);

  // Subnet 2: 200.168.10.64/27 should contain 200.168.10.75
  assert.equal(split.rows[2].networkStr, '200.168.10.64');
  assert.equal(split.rows[2].broadcastStr, '200.168.10.95');
  assert.equal(split.rows[2].firstUsableStr, '200.168.10.65');
  assert.equal(split.rows[2].lastUsableStr, '200.168.10.94');
  assert.equal(split.rows[2].containsInputIp, true);

  // Large split truncation test (e.g. 10 borrowed bits -> 1024 subnets)
  // Must be bounded without creating 1024 rows
  const largeSplit = splitSubnet('10.0.0.0', 8, 10, 8);
  assert.equal(largeSplit.totalSubnets, 1024);
  assert.equal(largeSplit.displayedSubnets, 8);
  assert.equal(largeSplit.isTruncated, true);
  assert.equal(largeSplit.rows.length, 8);
});

test('all worked examples evaluate consistently and match syllabus requirements', () => {
  assert.ok(WORKED_EXAMPLES.length >= 6, 'Must provide at least 6 worked examples');

  for (const ex of WORKED_EXAMPLES) {
    const calc = calculateSubnet(ex.inputIp, ex.inputPrefix);
    assert.ok(calc.networkStr, `Example ${ex.id} must produce network`);
    if (ex.borrowedBits) {
      const split = splitSubnet(ex.inputIp, ex.inputPrefix, ex.borrowedBits);
      assert.ok(split.rows.length > 0, `Split for example ${ex.id} must have rows`);
    }
  }
});

test('scope boundaries do not mistake special purpose space for public routability',()=>{
  for(const [address,scope] of [['0.0.0.0','this-host'],['0.255.255.255','this-host'],['1.0.0.0','unclassified'],['192.0.0.0','ietf-protocol'],['192.0.0.255','ietf-protocol'],['192.0.1.0','unclassified'],['100.63.255.255','unclassified'],['100.64.0.0','carrier-grade-nat'],['100.127.255.255','carrier-grade-nat'],['100.128.0.0','unclassified']] as const){
    assert.equal(calculateSubnet(address,24).scope,scope,address);
  }
});
