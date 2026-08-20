#!/usr/bin/env python3
"""Give the fly six legs it can actually move.

The supplied rig cannot drive them. Weighing every vertex against every bone
says why: the front-left leg is 59% weighted to `bone_7`, which is the *body*,
and the front-right is 99% weighted to `tripo::0_Left_Limb_6`, which also holds
most of the other five. Rotating any of it swings half the animal.

So the legs are re-skinned. The six are found geometrically — everything below
the body mass, clustered by proximity — and each is bound to a new bone of its
own, planted where that leg meets the thorax. Weight feathers in over the first
centimetre so the joint bends instead of tearing, and whatever a vertex does not
give the new bone it keeps on the body.

Nothing else is touched: same mesh, same materials, same wing bones, same
existing skeleton. Six nodes are appended and six columns of inverse bind
matrices with them.

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
# Weight feathers from the root outward over this distance, so the top of the
# leg still moves with the body and the joint does not tear open.
FEATHER = 0.085


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

    added = []
    for i, group in enumerate(clusters):
        # The root of a leg is where it meets the body: the highest tenth of it.
        top = sorted(group, key=lambda k: -pos[k][1])[: max(6, len(group) // 10)]
        root = [sum(pos[k][d] for k in top) / len(top) for d in range(3)]
        row = "front middle rear".split()[i // 2]
        side = "left" if root[2] < 0 else "right"
        name = f"leg_{row}_{side}"

        nodes.append({"name": name, "translation": apply(to_local, root)})
        index = len(nodes) - 1
        nodes[anchor].setdefault("children", []).append(index)
        skin["joints"].append(index)
        added.append((name, index, root, group))

    # -- re-skin -------------------------------------------------------------
    slot = {node: k for k, (_, node, _, _) in enumerate(added)}
    body_slot = skin["joints"].index(body)
    moved = 0
    for name, node, root, group in added:
        mine = skin["joints"].index(node)
        for k in group:
            p = pos[k]
            far = sum((p[d] - root[d]) ** 2 for d in range(3)) ** 0.5
            w = min(1.0, max(0.0, far / FEATHER))
            w = w * w * (3 - 2 * w)
            _, at_j, fj, nj = joints[k]
            _, at_w, fw, nw = weights[k]
            struct.pack_into("<" + fj * nj, bn, at_j, mine, body_slot, 0, 0)
            struct.pack_into("<" + fw * nw, bn, at_w, w, 1.0 - w, 0.0, 0.0)
            moved += 1
        print(f"  {name:20s} root ({root[0]:+.2f},{root[1]:+.2f},{root[2]:+.2f})  {len(group)} vertices")

    # -- inverse bind matrices for the new bones -----------------------------
    old = accessor(js, bn, skin["inverseBindMatrices"])
    mats = [list(v[0]) for v in old]
    for _, node, root, _ in added:
        mats.append(invert(world(node)))

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
    print(f"wrote {dst}: {len(added)} new leg bones, {moved} vertices re-skinned")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
