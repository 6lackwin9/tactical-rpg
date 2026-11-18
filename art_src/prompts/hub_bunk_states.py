#!/usr/bin/env python3
# AI-17 bunk states (Tushar 2026-11-18: cut from the hub design reference, not the queue sheet). Input DIR/b2.png =
# a 68 x 130 crop (462,470)-(530,600) of Drive flow/out/Region Maps/Underground_vault_..._ce4b19f9.jpg (bunk 2); writes
# DIR/kiln_bunk_{empty,benched,active,crypt}.png (80 x 80) and a review sheet.
#   python art_src/prompts/hub_bunk_states.py DIR
import sys, random
from PIL import Image, ImageDraw, ImageFilter, ImageEnhance
S=sys.argv[1]
src=Image.open(S+"/b2.png").convert("RGBA")   # 68 x 130 cut of bunk 2 from the hub design reference
W,H=src.size
BX0,BX1,BTOP,BLANK1,FOOT1=14,52,47,103,128
def mask():
    m=Image.new("L",(W,H),0); d=ImageDraw.Draw(m)
    d.rectangle([2,0,W-3,BTOP],fill=255); d.rectangle([BX0,BTOP,BX1,FOOT1],fill=255); return m
rnd=random.Random(1913)
def plate(im):
    d=ImageDraw.Draw(im); d.rectangle([27,38,40,43],fill=(52,40,26,255)); d.rectangle([28,39,39,42],fill=(150,118,60,255)); d.line([28,39,39,39],fill=(190,156,88,255))
def empty(im):
    px=im.load()
    straw=[px[x,y] for x in range(BX0+3,BX1-3) for y in range(106,115)]
    for y in range(BTOP+2,BLANK1+1):
        for x in range(BX0+2,BX1-1):
            r,g,b,a=rnd.choice(straw); px[x,y]=(int(r*.78),int(g*.76),int(b*.74),255)
    reg=im.crop((BX0+2,BTOP+2,BX1-1,BLANK1+1)).filter(ImageFilter.GaussianBlur(0.7)); im.paste(reg,(BX0+2,BTOP+2))
    d=ImageDraw.Draw(im)   # the rolled grey mattress tied at the head
    d.rounded_rectangle([BX0+3,BTOP+3,BX1-3,BTOP+14],radius=5,fill=(70,72,76,255))
    d.rounded_rectangle([BX0+4,BTOP+4,BX1-4,BTOP+12],radius=4,fill=(118,120,124,255))
    d.line([BX0+6,BTOP+6,BX1-6,BTOP+6],fill=(150,152,156,255))
    for cx in (BX0+11,BX1-11):
        d.line([cx,BTOP+3,cx,BTOP+14],fill=(92,66,40,255),width=2)
    d.ellipse([BX0+2,BTOP+4,BX0+7,BTOP+13],fill=(96,98,102,255),outline=(54,56,60,255))
def active(im):
    plate(im); d=ImageDraw.Draw(im)   # packed canvas kit bag and a strapped bedroll at the foot
    d.rounded_rectangle([BX0+3,98,BX1-3,108],radius=4,fill=(58,46,34,255))
    d.rounded_rectangle([BX0+4,99,BX1-4,106],radius=3,fill=(110,96,72,255))
    for cx in (BX0+10,BX1-10): d.line([cx,98,cx,108],fill=(60,40,24,255),width=2)
    d.rounded_rectangle([BX0+13,88,BX1-11,104],radius=4,fill=(52,40,28,255))
    d.rounded_rectangle([BX0+14,89,BX1-12,102],radius=3,fill=(128,108,78,255))
    d.rectangle([BX0+14,89,BX1-12,92],fill=(104,86,60,255))
    d.line([BX0+21,92,BX0+21,102],fill=(70,50,30,255),width=2)
    d.rectangle([BX0+20,95,BX0+22,97],fill=(150,120,64,255))
def crypt(im):
    plate(im); px=im.load()
    for y in range(BTOP+1,BLANK1+4):   # pale burial cloth over the whole bed, a little past its sides
        for x in range(BX0,BX1+1):
            r,g,b,a=px[x,y]; l=(r+g+b)/3
            v=int(128+ (l-90)*0.5); v=max(100,min(168,v))
            px[x,y]=(v,v-3,v-8,255)
    d=ImageDraw.Draw(im)
    for x in (BX0-1,BX1+1):
        d.line([x,BTOP+6,x,BLANK1+2],fill=(120,118,112,255))
    for i,(a,b) in enumerate(((24,70),(29,68),(33,71))):   # dried flower stems laid on it
        d.line([a,b,a+12,b+10],fill=(86,66,44,255),width=1)
        d.point([(a,b),(a+1,b-1),(a-1,b)],fill=(120,70,60,255))
def benched(im): plate(im)
out={}
for name,fn in (("empty",empty),("benched",benched),("active",active),("crypt",crypt)):
    im=src.copy(); fn(im); im.putalpha(mask())
    small=im.resize((42,80),Image.LANCZOS)
    a=small.getchannel("A").point(lambda v:255 if v>140 else 0); small.putalpha(a)
    can=Image.new("RGBA",(80,80),(0,0,0,0)); can.alpha_composite(small,(19,0))
    can.save(f"{S}/kiln_bunk_{name}.png"); out[name]=can
sheet=Image.new("RGBA",(4*84*4,80*4),(40,40,40,255))
for i,n in enumerate(out): sheet.alpha_composite(out[n].resize((320,320),Image.NEAREST),(i*336,0))
sheet.save(S+"/states_sheet.png")
