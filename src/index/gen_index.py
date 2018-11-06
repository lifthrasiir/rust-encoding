# This is a part of rust-encoding.
# Copyright (c) 2013-2015, Kang Seonghoon.
# See README.md and LICENSE.txt for details.

import urllib
import sys
import os.path

CC0_LICENSE = "rust-encoding by Kang Seonghoon

To the extent possible under law, the person who associated CC0 with
rust-encoding has waived all copyright and related or neighboring rights
to rust-encoding.

You should have received a copy of the CC0 legalcode along with this
work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
"

def whatwg_index(name, comments):
    for line in urllib.urlopen('http://encoding.spec.whatwg.org/index-%s.txt' % name):
        line = line.strip()
        if not line: continue
        if line.startswith('#'):
            comments.append('//' + line[1:])
            continue
        parts = line.split(None, 2)
        key = int(parts[0], 0)
        value = int(parts[1], 0)
        yield key, value

def mkdir_and_open(crate, name):
    dirname = os.path.join(os.path.dirname(__file__), crate)
    try:
        os.mkdir(dirname)
    except Exception:
        pass
    return open(os.path.join(dirname, '%s.rs' % name.replace('-', '_')), 'wb')

def write_header(f, name, comments):
    print >>f, '// AUTOGENERATED FROM index-%s.txt, ORIGINAL COMMENT FOLLOWS:' % name
    print >>f, '//'
    for line in comments:
        print >>f, line

def write_comma_separated(f, prefix, l, width=80):
    buffered = ''
    for i in l:
        i = str(i)
        if len(prefix) + len(buffered) + len(i) <= width:
            buffered += i
        else:
            print >>f, prefix + buffered.rstrip()
            buffered = i
    if buffered:
        print >>f, prefix + buffered.rstrip()

def make_minimal_trie(invdata, lowerlimit=0x10000):
    maxvalue = max(invdata.keys()) + 1
    best = 0xffffffff
    besttrie = None
    for triebits in xrange(21):
        lower = [None] * (1<<triebits)
        upper = []
        lowermap = {tuple(lower): 0}
        for i in xrange(0, maxvalue, 1<<triebits):
            blk = [invdata.get(j) for j in xrange(i, i + (1<<triebits))]
            loweridx = lowermap.get(tuple(blk))
            if loweridx is None:
                loweridx = len(lower)
                lowermap[tuple(blk)] = loweridx
                lower += blk
            upper.append(loweridx)
        if len(lower) < lowerlimit and best >= len(lower) + len(upper):
            best = len(lower) + len(upper)
            besttrie = (triebits, lower, upper)
    return besttrie

def generate_single_byte_index(crate, name):
    modname = name.replace('-', '_')

    data = [None] * 128
    invdata = {}
    comments = []
    for key, value in whatwg_index(name, comments):
        assert 0 <= key < 128 and 0 <= value < 0xffff and data[key] is None and value not in invdata
        data[key] = value
        invdata[value] = key

    # generate a trie with a minimal amount of data
    triebits, lower, upper = make_minimal_trie(invdata, lowerlimit=0x10000)

    with mkdir_and_open(crate, name) as f:
        write_header(f, name, comments)
        print >>f
        print >>f, "static FORWARD_TABLE: &'static [u16] = &["
        write_comma_separated(f, '    ',
            ['%d, ' % (0xffff if value is None else value) for value in data])
        print >>f, '];'
        print >>f
        print >>f, '/// Returns the index code point for pointer `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn forward(code: u8) -> u16 {'
        print >>f, '    FORWARD_TABLE[(code - 0x80) as usize]'
        print >>f, '}'
        print >>f
        print >>f, "static BACKWARD_TABLE_LOWER: &'static [u8] = &["
        write_comma_separated(f, '    ', ['%d, ' % (0 if v is None else v+0x80) for v in lower])
        print >>f, '];'
        print >>f
        print >>f, "static BACKWARD_TABLE_UPPER: &'static [u16] = &["
        write_comma_separated(f, '    ', ['%d, ' % v for v in upper])
        print >>f, '];'
        print >>f
        print >>f, '/// Returns the index pointer for code point `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn backward(code: u32) -> u8 {'
        print >>f, '    let offset = (code >> %d) as usize;' % triebits
        print >>f, '    let offset = if offset < %d {BACKWARD_TABLE_UPPER[offset] as usize} else {0};' % len(upper)
        print >>f, '    BACKWARD_TABLE_LOWER[offset + ((code & %d) as usize)]' % ((1<<triebits)-1)
        print >>f, '}'
        print >>f
        print >>f, '#[cfg(test)]'
        print >>f, 'single_byte_tests!('
        print >>f, '    mod = %s' % modname
        print >>f, ');'

    return 2 * len(data) + len(lower) + 2 * len(upper)

def generate_multi_byte_index(crate, name):
    modname = name.replace('-', '_')

    data = {}
    invdata = {}
    dups = []
    comments = []
    morebits = False
    for key, value in whatwg_index(name, comments):
        assert 0 <= key < 0xffff and 0 <= value < 0x110000 and value != 0xffff and key not in data
        if value >= 0x10001:
            assert (value >> 16) == 2
            morebits = True
        data[key] = value
        if value not in invdata:
            invdata[value] = key
        else:
            dups.append(key)

    # Big5 has four two-letter forward mappings, we use special entries for them
    if name == 'big5':
        specialidx = [1133, 1135, 1164, 1166]
        assert all(key not in data for key in specialidx)
        assert all(value not in invdata for value in xrange(len(specialidx)))
        for value, key in enumerate(specialidx):
            data[key] = value
            dups.append(key) # no consistency testing for them

    # generate a trie with a minimal amount of data
    triebits, lower, upper = make_minimal_trie(invdata, lowerlimit=0x10000)

    # JIS X 0208 index has two ranges [8272,8836) and [8836,11280) to support two slightly
    # different encodings EUC-JP and Shift_JIS; the default backward function would favor
    # the former, so we need a separate mapping for the latter.
    #
    # fortunately for us, all allocated codes in [8272,8836) have counterparts in others,
    # so we only need a smaller remapping from [8272,8836) to other counterparts.
    remap = None
    if name == 'jis0208':
        REMAP_MIN = 8272
        REMAP_MAX = 8835

        invdataminusremap = {}
        for key, value in data.items():
            if value not in invdataminusremap and not REMAP_MIN <= key <= REMAP_MAX:
                invdataminusremap[value] = key

        remap = []
        for i in xrange(REMAP_MIN, REMAP_MAX+1):
            if i in data:
                assert data[i] in invdataminusremap
                value = invdataminusremap[data[i]]
                assert value < 0x10000
                remap.append(value)
            else:
                remap.append(0xffff)

    minkey = min(data)
    maxkey = max(data) + 1
    with mkdir_and_open(crate, name) as f:
        write_header(f, name, comments)
        print >>f
        print >>f, "static FORWARD_TABLE: &'static [u16] = &["
        write_comma_separated(f, '    ',
            ['%d, ' % (data.get(key, 0xffff) & 0xffff) for key in xrange(minkey, maxkey)])
        print >>f, '];'
        if morebits:
            print >>f
            print >>f, "static FORWARD_TABLE_MORE: &'static [u32] = &["
            bits = []
            for i in xrange(minkey, maxkey, 32):
                v = 0
                for j in xrange(32):
                    v |= (data.get(i+j, 0) >= 0x10000) << j
                bits.append(v)
            write_comma_separated(f, '    ', ['%d, ' % v for v in bits])
            print >>f, '];'
        print >>f
        print >>f, '/// Returns the index code point for pointer `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn forward(code: u16) -> u32 {'
        if minkey != 0:
            print >>f, '    let code = (code as usize).wrapping_sub(%d);' % minkey
        else:
            print >>f, '    let code = code as usize;'
        print >>f, '    if code < %d {' % (maxkey - minkey)
        if morebits:
            print >>f, '        (FORWARD_TABLE[code] as u32) | ' + \
                               '(((FORWARD_TABLE_MORE[code >> 5] >> (code & 31)) & 1) << 17)'
        else:
            print >>f, '        FORWARD_TABLE[code] as u32'
        print >>f, '    } else {'
        print >>f, '        0xffff'
        print >>f, '    }'
        print >>f, '}'
        print >>f
        print >>f, "static BACKWARD_TABLE_LOWER: &'static [u16] = &["
        write_comma_separated(f, '    ', ['%d, ' % (0xffff if v is None else v) for v in lower])
        print >>f, '];'
        print >>f
        print >>f, "static BACKWARD_TABLE_UPPER: &'static [u16] = &["
        write_comma_separated(f, '    ', ['%d, ' % v for v in upper])
        print >>f, '];'
        if remap:
            print >>f
            print >>f, "static BACKWARD_TABLE_REMAPPED: &'static [u16] = &["
            write_comma_separated(f, '    ', ['%d, ' % v for v in remap])
            print >>f, '];'
        print >>f
        print >>f, '/// Returns the index pointer for code point `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn backward(code: u32) -> u16 {'
        print >>f, '    let offset = (code >> %d) as usize;' % triebits
        print >>f, '    let offset = if offset < %d {BACKWARD_TABLE_UPPER[offset] as usize} else {0};' % len(upper)
        print >>f, '    BACKWARD_TABLE_LOWER[offset + ((code & %d) as usize)]' % ((1<<triebits)-1)
        print >>f, '}'
        if remap:
            print >>f
            assert name == 'jis0208'
            print >>f, '/// Returns the index shift_jis pointer for code point `code`.'
            print >>f, '#[inline]'
            print >>f, 'pub fn backward_remapped(code: u32) -> u16 {'
            print >>f, '    let value = backward(code);'
            print >>f, '    if %d <= value && value <= %d {' % (REMAP_MIN, REMAP_MAX)
            print >>f, '        BACKWARD_TABLE_REMAPPED[(value - %d) as usize]' % REMAP_MIN
            print >>f, '    } else {'
            print >>f, '        value'
            print >>f, '    }'
            print >>f, '}'
        print >>f
        print >>f, '#[cfg(test)]'
        print >>f, 'multi_byte_tests!('
        print >>f, '    mod = %s,' % modname
        if remap:
            print >>f, '    remap = [%d, %d],' % (REMAP_MIN, REMAP_MAX)
        if dups:
            print >>f, '    dups = ['
            write_comma_separated(f, '        ', ['%d, ' % v for v in sorted(dups)])
            print >>f, '    ]'
        else:
            print >>f, '    dups = []'
        print >>f, ');'

    tablesz = 2 * (maxkey - minkey) + 2 * len(lower) + 2 * len(upper)
    if morebits: tablesz += 4 * ((maxkey - minkey + 31) // 32)
    if remap: tablesz += 2 * len(remap)
    return tablesz

def generate_multi_byte_range_lbound_index(crate, name):
    modname = name.replace('-', '_')

    data = []
    comments = []
    for key, value in whatwg_index(name, comments):
        data.append((key, value))
    assert data and data == sorted(data)

    minkey, minvalue = data[0]
    maxkey, maxvalue = data[-1]
    if data[0] != (0, 0):
        data.insert(0, (0, 0))
    maxlog2 = 0
    while 2**(maxlog2 + 1) <= len(data):
        maxlog2 += 1

    if name == 'gb18030-ranges':
        keyubound = 0x110000
        valueubound = 126 * 10 * 126 * 10
    else:
        keyubound = maxkey + 1
        valueubound = maxvalue + 1

    with mkdir_and_open(crate, name) as f:
        write_header(f, name, comments)
        print >>f
        print >>f, "static FORWARD_TABLE: &'static [u32] = &["
        write_comma_separated(f, '    ', ['%d, ' % value for key, value in data])
        print >>f, '];'
        print >>f
        print >>f, "static BACKWARD_TABLE: &'static [u32] = &["
        write_comma_separated(f, '    ', ['%d, ' % key for key, value in data])
        print >>f, '];'
        print >>f
        print >>f, '/// Returns the index code point for pointer `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn forward(code: u32) -> u32 {'
        if minkey > 0:
            print >>f, '    if code < %d { return 0xffffffff; }' % minkey
        if name == 'gb18030-ranges': # has "invalid" region inside
            print >>f, '    if (code > 39419 && code < 189000) || code > 1237575 { return 0xffffffff; }'
        print >>f, '    let mut i = if code >= BACKWARD_TABLE[%d] {%d} else {0};' % \
                (2**maxlog2 - 1, len(data) - 2**maxlog2 + 1)
        for i in xrange(maxlog2-1, -1, -1):
            print >>f, '    if code >= BACKWARD_TABLE[i%s] { i += %d; }' % \
                    ('+%d' % (2**i-1) if i > 0 else '', 2**i)
        print >>f, '    (code - BACKWARD_TABLE[i-1]) + FORWARD_TABLE[i-1]'
        print >>f, '}'
        print >>f
        print >>f, '/// Returns the index pointer for code point `code` in this index.'
        print >>f, '#[inline]'
        print >>f, 'pub fn backward(code: u32) -> u32 {'
        if minvalue > 0:
            print >>f, '    if code < %d { return 0xffffffff; }' % minvalue
        print >>f, '    let mut i = if code >= FORWARD_TABLE[%d] {%d} else {0};' % \
                (2**maxlog2 - 1, len(data) - 2**maxlog2 + 1)
        for i in xrange(maxlog2-1, -1, -1):
            print >>f, '    if code >= FORWARD_TABLE[i%s] { i += %d; }' % \
                    ('+%d' % (2**i-1) if i > 0 else '', 2**i)
        print >>f, '    (code - FORWARD_TABLE[i-1]) + BACKWARD_TABLE[i-1]'
        print >>f, '}'
        print >>f
        print >>f, '#[cfg(test)]'
        print >>f, 'multi_byte_range_tests!('
        print >>f, '    mod = %s,' % modname
        print >>f, '    key = [%d, %d], key < %d,' % (minkey, maxkey, keyubound)
        print >>f, '    value = [%d, %d], value < %d' % (minvalue, maxvalue, valueubound)
        print >>f, ');'

    return 8 * len(data)

INDICES = {
    'singlebyte/ibm866':          generate_single_byte_index,
    'singlebyte/iso-8859-2':      generate_single_byte_index,
    'singlebyte/iso-8859-3':      generate_single_byte_index,
    'singlebyte/iso-8859-4':      generate_single_byte_index,
    'singlebyte/iso-8859-5':      generate_single_byte_index,
    'singlebyte/iso-8859-6':      generate_single_byte_index,
    'singlebyte/iso-8859-7':      generate_single_byte_index,
    'singlebyte/iso-8859-8':      generate_single_byte_index,
    'singlebyte/iso-8859-10':     generate_single_byte_index,
    'singlebyte/iso-8859-13':     generate_single_byte_index,
    'singlebyte/iso-8859-14':     generate_single_byte_index,
    'singlebyte/iso-8859-15':     generate_single_byte_index,
    'singlebyte/iso-8859-16':     generate_single_byte_index,
    'singlebyte/koi8-r':          generate_single_byte_index,
    'singlebyte/koi8-u':          generate_single_byte_index,
    'singlebyte/macintosh':       generate_single_byte_index,
    'singlebyte/windows-874':     generate_single_byte_index,
    'singlebyte/windows-1250':    generate_single_byte_index,
    'singlebyte/windows-1251':    generate_single_byte_index,
    'singlebyte/windows-1252':    generate_single_byte_index,
    'singlebyte/windows-1253':    generate_single_byte_index,
    'singlebyte/windows-1254':    generate_single_byte_index,
    'singlebyte/windows-1255':    generate_single_byte_index,
    'singlebyte/windows-1256':    generate_single_byte_index,
    'singlebyte/windows-1257':    generate_single_byte_index,
    'singlebyte/windows-1258':    generate_single_byte_index,
    'singlebyte/x-mac-cyrillic':  generate_single_byte_index,

    'tradchinese/big5':           generate_multi_byte_index,
    'korean/euc-kr':              generate_multi_byte_index,
    'simpchinese/gb18030':        generate_multi_byte_index,
    'japanese/jis0208':           generate_multi_byte_index,
    'japanese/jis0212':           generate_multi_byte_index,

    'simpchinese/gb18030-ranges': generate_multi_byte_range_lbound_index,
}

if __name__ == '__main__':
    import sys
    filter = sys.argv[1] if len(sys.argv) > 1 else ''
    for index, generate in INDICES.items():
        crate, _, index = index.partition('/')
        if filter not in index: continue
        print >>sys.stderr, 'generating index %s...' % index,
        tablesz = generate(crate, index)
        print >>sys.stderr, '%d bytes.' % tablesz
        write_license_file(crate)
