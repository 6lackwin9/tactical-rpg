#!/usr/bin/env python3
# Cuts image_pipeline outputs props_hub_bunk_states_2 (cells 5-6: talk marker, talk icon) and service_facility_1
# (the Facilities badge): magenta key, spill to amber glow or grey, fit to 64 x 80 / 32 x 32 / 32 x 32.
# Edit TR to the folder holding the two downloaded outputs.  python art_src/prompts/cut_queue_markers.py OUTDIR
import sys
from PIL import Image
S=sys.argv[1]; TR="/root/.claude/projects/-home-user/64fc4ad9-bf46-5cb4-b671-5400e8f4dea5/tool-results/"
def key(im):
    im=im.convert("RGBA"); px=im.load()
    for y in range(im.height):
        for x in range(im.width):
            r,g,b,a=px[x,y]
            if r>150 and b>150 and g<120 and abs(r-b)<90: px[x,y]=(0,0,0,0)
            elif r>140 and b>120 and g<110: px[x,y]=(r//2+20,g,b//2+20,255) if False else (0,0,0,0)
    for y in range(im.height):
        for x in range(im.width):
            r,g,b,a=px[x,y]
            if a and ((b>g+14 and r>g+14) or (r>195 and g<155 and b>40)):          # magenta spill: the glow becomes amber, a fringe becomes grey
                if r>170: px[x,y]=(230,140,40,150)
                else: m=(g+min(r,b))//2; px[x,y]=(m,m,m+4,255)
    return im
def fit(im,box,size,keep_h=None):
    c=key(im.crop(box)); bb=c.getchannel("A").getbbox(); c=c.crop(bb)
    W,H=size; s=min(W/c.width,H/c.height); n=(max(1,round(c.width*s)),max(1,round(c.height*s)))
    c=c.resize(n,Image.LANCZOS); a=c.getchannel("A").point(lambda v:0 if v<60 else (v if v<200 else 255)); c.putalpha(a)
    out=Image.new("RGBA",size,(0,0,0,0)); out.alpha_composite(c,((W-n[0])//2,H-n[1] if keep_h else (H-n[1])//2)); return out
sheet=Image.open(TR+"mcp-Google_Drive-blob-1795026368746-fod8ob.png")
badge=Image.open(TR+"mcp-Google_Drive-blob-1795026368573-28kwmw.png")
fit(sheet,(560,660,720,1015),(64,80),keep_h=True).save(S+"/bunk_candle.png")
fit(sheet,(770,760,990,960),(32,32)).save(S+"/bunk_talk.png")
fit(badge,(0,0,1024,1024),(32,32)).save(S+"/facility.png")
