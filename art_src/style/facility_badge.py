#!/usr/bin/env python3
# AI-19 stand-in: draws the 32 x 32 Facilities service badge (hammer over a timber frame) to the path given.
#   python art_src/style/facility_badge.py kits/rpg/assets/ui/services/facility.png
# A painted take is queued on image_pipeline (service_facility_1); it replaces this in place.
from PIL import Image, ImageDraw, ImageFilter
import sys
S=8; N=32*S
im=Image.new("RGBA",(N,N),(0,0,0,0)); d=ImageDraw.Draw(im)
WOOD=(98,62,38,255); WOOD_HI=(134,88,52,255); WOOD_LO=(66,40,26,255)
IRON=(70,72,82,255); IRON_HI=(122,126,136,255); IRON_LO=(40,42,50,255)
def P(x,y): return (x*S,y*S)
def beam(a,b,w,c):
    d.line([P(*a),P(*b)],fill=c,width=int(w*S))
# timber frame: A-frame truss with posts and a tie beam
beam((4,14),(16,4),3,WOOD); beam((28,14),(16,4),3,WOOD)
beam((6,13),(6,29),3,WOOD_LO); beam((26,13),(26,29),3,WOOD_LO)
beam((4,15),(28,15),3,WOOD)
beam((16,5),(16,15),2.4,WOOD_LO)
beam((7,28),(25,17),2.4,WOOD_LO)
# highlights
beam((5,13.4),(16,4.2),0.9,WOOD_HI); beam((5,14.2),(27,14.2),0.9,WOOD_HI)
# hammer over frame: handle diagonal, head top-right
beam((9,27),(21,12),2.6,WOOD_HI); beam((9,27),(21,12),1.0,(162,114,70,255))
head=[P(15,10),P(21,4.5),P(27,11),P(21,16.5)]
d.polygon([P(16,9),P(20,5),P(27.5,12),P(23.5,16)],fill=IRON)
d.polygon([P(16,9),P(20,5),P(21,6),P(17,10)],fill=IRON_HI)
d.polygon([P(23.5,16),P(27.5,12),P(27.5,13.5),P(24.5,16.5)],fill=IRON_LO)
d.line([P(19.5,6.5),P(25.5,12.5)],fill=IRON_HI,width=S)
small=im.resize((32,32),Image.BOX)
px=small.load()
for y in range(32):
  for x in range(32):
    r,g,b,a=px[x,y]; px[x,y]=(r,g,b,255) if a>110 else (0,0,0,0)
# 1px dark outline
out=small.copy(); op=out.load()
for y in range(32):
  for x in range(32):
    if px[x,y][3]==0 and any(0<=x+dx<32 and 0<=y+dy<32 and px[x+dx,y+dy][3] for dx,dy in((1,0),(-1,0),(0,1),(0,-1))):
      op[x,y]=(22,18,18,255)
out.save(sys.argv[1])
