#!/usr/bin/env python3
"""Give the fly six legs it can actually move.

The supplied rig cannot drive them. Weighing every vertex against every bone
says why: the front-left leg is 59% weighted to `bone_7`, which is the *body*,
and the front-right is 99% weighted to `tripo::0_Left_Limb_6`, which also holds
most of the other five. Rotating any of it swings half the animal.

So the legs are re-skinned. The six are found geometrically — everything below
the body mass, clustered by proximity — and each gets the three bones a fly's
leg actually has — femur, tibia, tarsus — planted along it in that order, so it
folds at two joints like an insect's rather than swinging as one stick.

Weight hands over between segments *across* each joint rather than at it, so the
mesh bends instead of creasing, and feathers into the body over the top of the
femur so the hip does not tear open. Four influences a vertex: body, femur,
tibia, tarsus, which is exactly the four glTF allows.

Nothing else is touched: same mesh, same materials, same wing bones, same
existing skeleton. Eighteen nodes are appended and eighteen columns of inverse
bind matrices with them.

    python3 tools/rig-the-legs.py in.glb out.glb
"""

import json
import struct
import sys

# Vertices this far below the body's mass are leg. Measured off the model: the
# thorax bottoms out around a fifth of the way up and the feet reach the floor.
LEG_TOP = 0.18
# Two vertices closer than this are the same leg. Loose enough to bridge the
# gaps between segments, tight enough that two legs never merge.
SAME_LEG = 0.059
# A cluster smaller than this is a toe or a stray shell, not a leg.
LEAST = 120
# Weight feathers into the body over this fraction of the leg's length, so the
# hip does not tear open when the femur swings.
#
# A fraction, not a distance. It was 0.085 in absolute units against legs only
# 0.17 to 0.23 long, so the feather covered the whole top segment and handed it
# all to the body: the lower leg animated and the upper sat perfectly still.
# The tool prints where the weight landed now, which is the check that catches
# it without anybody having to notice by eye.
HIP = 0.16
# Where the two joints sit, as fractions of root-to-foot: femur/tibia, then
# tibia/tarsus. A fly's leg has three segments and the tarsus is the short
# jointed foot on the end of it.
JOINTS = (0.38, 0.74)
# How much of the leg's length each handover is spread over.
BLEND = 0.20
# How far *inboard of the leg* the femur's pivot is planted, as a fraction of
# the leg's length.
#
# This is the difference between a leg that swings and a leg whose top half
# looks nailed on. A hinge cannot move the geometry sitting on its own axis, so
# a femur bone placed exactly at the leg root pins the top of the leg in place
# and swings only the far end — which is precisely how it looked, and measuring
# the drive angle proved the bone was turning a full twenty-five degrees while
# doing it. Real insects have a coxa for this: the pivot is inboard, in the
# body, and the whole visible leg swings from it.
COXA = 0.34


def load(path):
    raw = open(path, "rb").read()
    _, _, _ = struct.unpack_from("<III", raw, 0)
    off, js, bn = 12, None, None
    while off < len(raw):
        length, kind = struct.unpack_from("<II", raw, off)
        body = raw[off + 8 : off + 8 + length]
        if kind == 0x4E4F534A:
            js = json.loads(body)
        elif kind == 0x004E4942:
            bn = bytearray(body)
        off += 8 + length
    return js, bn


def accessor(js, bn, i):
    a = js["accessors"][i]
    v = js["bufferViews"][a["bufferView"]]
    base = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    n = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[a["type"]]
    fmt = {5126: "f", 5123: "H", 5125: "I", 5121: "B"}[a["componentType"]]
    size = struct.calcsize(fmt)
    stride = v.get("byteStride") or n * size
    return [
        (struct.unpack_from("<" + fmt * n, bn, base + k * stride), base + k * stride, fmt, n)
        for k in range(a["count"])
    ]


def mul(a, b):
    return [sum(a[r + k * 4] * b[k + c * 4] for k in range(4)) for c in range(4) for r in range(4)]


def trs(node):
    t = node.get("translation", [0, 0, 0])
    q = node.get("rotation", [0, 0, 0, 1])
    s = node.get("scale", [1, 1, 1])
    x, y, z, w = q
    r = [
        1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w), 0,
        2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w), 0,
        2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y), 0,
        0, 0, 0, 1,
    ]
    for c in range(3):
        for row in range(3):
            r[row + c * 4] *= s[c]
    r[12], r[13], r[14] = t
    return r


def invert(m):
    """Gauss-Jordan. General enough not to assume the matrix is rigid."""
    a = [[m[r + c * 4] for c in range(4)] + [1.0 if r == c else 0.0 for c in range(4)] for r in range(4)]
    for col in range(4):
        pivot = max(range(col, 4), key=lambda r: abs(a[r][col]))
        a[col], a[pivot] = a[pivot], a[col]
        d = a[col][col]
        a[col] = [v / d for v in a[col]]
        for r in range(4):
            if r != col and a[r][col]:
                f = a[r][col]
                a[r] = [v - f * w for v, w in zip(a[r], a[col])]
    return [a[r][4 + c] for c in range(4) for r in range(4)]


def apply(m, p):
    return [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]


def main(src, dst):
    js, bn = load(src)
    prim = js["meshes"][0]["primitives"][0]
    pos = [v[0] for v in accessor(js, bn, prim["attributes"]["POSITION"])]
    joints = accessor(js, bn, prim["attributes"]["JOINTS_0"])
    weights = accessor(js, bn, prim["attributes"]["WEIGHTS_0"])

    nodes = js["nodes"]
    skin = js["skins"][0]
    names = {n.get("name"): i for i, n in enumerate(nodes) if n.get("name")}

    parent = {}
    for i, n in enumerate(nodes):
        for c in n.get("children", []):
            parent[c] = i

    def world(i):
        m = trs(nodes[i])
        k = parent.get(i)
        while k is not None:
            m = mul(trs(nodes[k]), m)
            k = parent.get(k)
        return m

    # The middle of the body, for planting the leg pivots inboard of the legs.
    core = [
        sum(p[d] for p in pos if p[1] >= LEG_TOP) / max(1, sum(1 for p in pos if p[1] >= LEG_TOP))
        for d in range(3)
    ]

    # -- find the six legs ---------------------------------------------------
    low = [k for k, p in enumerate(pos) if p[1] < LEG_TOP]
    unseen, clusters = set(low), []
    while unseen:
        seed = unseen.pop()
        group, edge = [seed], [seed]
        while edge:
            here = pos[edge.pop()]
            near = [
                k
                for k in unseen
                if (pos[k][0] - here[0]) ** 2 + (pos[k][1] - here[1]) ** 2 + (pos[k][2] - here[2]) ** 2
                < SAME_LEG * SAME_LEG
            ]
            for k in near:
                unseen.discard(k)
                group.append(k)
                edge.append(k)
        if len(group) >= LEAST:
            clusters.append(group)
    if len(clusters) != 6:
        print(f"expected six legs, clustered {len(clusters)} — refusing to guess")
        return 1

    # Front to back, then left to right, so the names mean something.
    clusters.sort(key=lambda g: (-sum(pos[k][0] for k in g) / len(g), sum(pos[k][2] for k in g) / len(g)))

    body = names["bone_7"]
    anchor = names["tripo::Root"]
    to_local = invert(world(anchor))

    anchor_world = world(anchor)
    added = []
    for i, group in enumerate(clusters):
        # The root of a leg is where it meets the body: the highest tenth of it.
        # The foot is the lowest.
        by_height = sorted(group, key=lambda k: -pos[k][1])
        take = max(6, len(group) // 10)
        root = [sum(pos[k][d] for k in by_height[:take]) / take for d in range(3)]

        # How far down the leg each vertex is, measured *from the root* rather
        # than projected onto a straight line to the foot.
        #
        # A fly's leg is bent. Projected onto the chord, more than half the
        # vertices land past the last joint and the femur ends up with seven
        # per cent of the weight — which is exactly what happened, and it looks
        # like a leg whose top segment does not move. Distance from the root
        # follows the bend.
        far = {k: sum((pos[k][d] - root[d]) ** 2 for d in range(3)) ** 0.5 for k in group}
        # Ranked, not scaled. Scaling by the furthest vertex still gave the
        # tarsus half the weight, because the mesh is denser at the foot than
        # along the femur — the *distance* was fair and the *vertex count* was
        # not. Ranking makes the bands equal shares of the leg's geometry,
        # which is what decides whether a bone can be seen to move.
        order = sorted(group, key=lambda k: far[k])
        down = {k: i / max(1, len(order) - 1) for i, k in enumerate(order)}
        length = max(far.values()) or 1.0

        row = "front middle rear".split()[i // 2]
        side = "left" if root[2] < 0 else "right"
        stem = f"leg_{row}_{side}"

        # Each joint goes on the centroid of the vertices that sit at that
        # depth, so the chain follows the leg round its bend instead of cutting
        # the corner.
        def joint_at(depth):
            band = [k for k in group if abs(down[k] - depth) < 0.07]
            if not band:
                band = sorted(group, key=lambda k: abs(down[k] - depth))[:8]
            return [sum(pos[k][d] for k in band) / len(band) for d in range(3)]

        chain = []
        parent_index = anchor
        parent_world = anchor_world
        # The femur's pivot goes inboard of the leg, toward the middle of the
        # body, so the whole leg swings from it rather than pivoting on its own
        # shoulder.
        inboard = [core[d] - root[d] for d in range(3)]
        reach = sum(v * v for v in inboard) ** 0.5 or 1.0
        hip = [root[d] + inboard[d] / reach * (length * COXA) for d in range(3)]

        for part, at in (("femur", 0.0), ("tibia", JOINTS[0]), ("tarsus", JOINTS[1])):
            here = hip if at == 0.0 else joint_at(at)
            nodes.append(
                {"name": f"{stem}_{part}", "translation": apply(invert(parent_world), here)}
            )
            index = len(nodes) - 1
            nodes[parent_index].setdefault("children", []).append(index)
            skin["joints"].append(index)
            # The parent map has to grow with the tree. It did not, and `world()`
            # then computed a segment's bind matrix without its own parents in
            # it — the legs came out splayed flat on the floor.
            parent[index] = parent_index
            parent_world = mul(parent_world, trs(nodes[index]))
            parent_index = index
            chain.append(index)

        added.append((stem, chain, down, group))

    # -- re-skin -------------------------------------------------------------
    body_slot = skin["joints"].index(body)
    moved = 0
    for stem, chain, depth, group in added:
        slots = [skin["joints"].index(b) for b in chain]
        share = [0.0, 0.0, 0.0, 0.0]
        for k in group:
            down = depth[k]

            def ramp(at, down=down):
                t = min(1.0, max(0.0, (down - at + BLEND * 0.5) / BLEND))
                return t * t * (3 - 2 * t)

            below_knee = ramp(JOINTS[0])
            below_ankle = ramp(JOINTS[1])
            # Three segments, summing to one, with the handovers spread across
            # each joint rather than falling on it.
            parts = [
                1.0 - below_knee,
                below_knee * (1.0 - below_ankle),
                below_knee * below_ankle,
            ]
            hold = min(1.0, max(0.0, down / HIP))
            hold = hold * hold * (3 - 2 * hold)

            w = [v * hold for v in parts]
            w_body = 1.0 - hold
            _, at_j, fj, nj = joints[k]
            _, at_w, fw, nw = weights[k]
            struct.pack_into("<" + fj * nj, bn, at_j, slots[0], slots[1], slots[2], body_slot)
            struct.pack_into("<" + fw * nw, bn, at_w, w[0], w[1], w[2], w_body)
            for d in range(3):
                share[d] += w[d]
            share[3] += w_body
            moved += 1
        total = sum(share) or 1.0
        print(
            f"  {stem:18s} {len(group):4d} verts"
            f"   femur {100 * share[0] / total:4.0f}%"
            f"  tibia {100 * share[1] / total:4.0f}%"
            f"  tarsus {100 * share[2] / total:4.0f}%"
            f"  body {100 * share[3] / total:4.0f}%"
        )
        for name, part in zip(("femur", "tibia", "tarsus"), share[:3]):
            if part / total < 0.08:
                print(f"      ^ the {name} has almost no weight: it will not appear to move")

    # -- inverse bind matrices for the new bones -----------------------------
    old = accessor(js, bn, skin["inverseBindMatrices"])
    mats = [list(v[0]) for v in old]
    for _, chain, _, _ in added:
        for bone in chain:
            mats.append(invert(world(bone)))

    payload = b"".join(struct.pack("<16f", *m) for m in mats)
    while len(bn) % 4:
        bn.append(0)
    start = len(bn)
    bn.extend(payload)
    js["bufferViews"].append({"buffer": 0, "byteOffset": start, "byteLength": len(payload)})
    js["accessors"].append(
        {
            "bufferView": len(js["bufferViews"]) - 1,
            "componentType": 5126,
            "count": len(mats),
            "type": "MAT4",
        }
    )
    skin["inverseBindMatrices"] = len(js["accessors"]) - 1
    js["buffers"][0]["byteLength"] = len(bn)

    # -- write ---------------------------------------------------------------
    blob = json.dumps(js, separators=(",", ":")).encode()
    blob += b" " * ((4 - len(blob) % 4) % 4)
    while len(bn) % 4:
        bn.append(0)
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(blob) + 8 + len(bn))
    out += struct.pack("<II", len(blob), 0x4E4F534A) + blob
    out += struct.pack("<II", len(bn), 0x004E4942) + bytes(bn)
    open(dst, "wb").write(out)
    print(f"wrote {dst}: {3 * len(added)} new leg bones, {moved} vertices re-skinned")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
