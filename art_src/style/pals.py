# Candidate sprite palettes for the style-lock board. Slots are the rig's base keys; each ships a 3-tone ramp.
KAELEN_B = dict(metal='#7d8592', fore='#b8a98e', sleeve='#b8a98e', main='#7a2a22', accent='#7a2a22', leg='#454a54',
                leather='#5e3d26', glove='#5e3d26', boot='#4a3222', skin='#c09070', hair='#c9a24e', wood='#6e5034',
                shield='#4d5560', emblem='#b8a98e', trim='#a08850')
WATCH_B = dict(metal='#6e7580', fore='#6e7580', sleeve='#6a4430', main='#6e2420', accent='#6e2420', leg='#3c4049',
               leather='#6a4430', glove='#4a3222', boot='#4a3222', skin='#b08566', hair='#3a2c24', wood='#6e5034',
               shield='#3c4049', emblem='#8a7a60', trim='#8a7a60')
def darker(p, k=0.8):
    out = {}
    for s, h in p.items():
        c = [int(h[i:i+2], 16) for i in (1, 3, 5)]
        g = sum(c) / 3
        c = [round((x * 0.8 + g * 0.2) * k) for x in c]   # desaturate 20 %, darken
        out[s] = '#%02x%02x%02x' % tuple(c)
    return out
KAELEN_C, WATCH_C = darker(KAELEN_B), darker(WATCH_B)
