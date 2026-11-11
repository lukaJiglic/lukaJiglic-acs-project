"""The sizes and index ranges of docs/encoding.md (encoding version 1) as constants.

tests/test_spec007.py parses the tables of docs/encoding.md and fails if any
number here differs (spec 007, criterion 1). Change the document first.
"""

ENCODING_VERSION = 1

N_CARDS = 54

# --- Section 3: tokens -----------------------------------------------------

MAX_TOKENS = 128  # sequence limit
TOKEN_FIELDS = ("event", "seat", "trick", "position in trick", "index")
# Number of values of each token field, in the order of TOKEN_FIELDS.
TOKEN_FIELD_SIZES = (288, 5, 13, 5, 128)
N_EVENTS = TOKEN_FIELD_SIZES[0]
NO_SEAT = 4  # seat field of an event without a seat

# name: (first id, count)
EVENT_GROUPS = {
    "PAD": (0, 1),
    "BOS": (1, 1),
    "BID": (2, 13),
    "CONTRACT": (15, 13),
    "KING": (28, 4),
    "TALON_SHOWN": (32, 54),
    "TALON_TAKEN": (86, 6),
    "DISCARD_SHOWN": (92, 54),
    "ANNOUNCE": (146, 5),
    "KONTRA": (151, 6),
    "ANNOUNCE_DONE": (157, 1),
    "ANNOUNCE_BARVNI_VALAT": (158, 1),
    "PLAY": (159, 54),
    "KLOP_TALON": (213, 54),
    "RESERVED": (267, 21),
}

# --- Section 4: state token --------------------------------------------------

STATE_FEATURES = 1382

# name: (offset, width)
STATE_BLOCKS = {
    "A1": (0, 54),
    "A2": (54, 216),
    "A3": (270, 162),
    "A4": (432, 54),
    "A5": (486, 54),
    "A6": (540, 54),
    "A7": (594, 54),
    "A8": (648, 54),
    "B1": (702, 8),
    "B2": (710, 13),
    "B3": (723, 4),
    "B4": (727, 4),
    "B5": (731, 4),
    "B6": (735, 4),
    "B7": (739, 4),
    "B8": (743, 8),
    "B9": (751, 4),
    "C1": (755, 13),
    "C2": (768, 4),
    "C3": (772, 4),
    "C4": (776, 52),
    "D1": (828, 10),
    "D2": (838, 30),
    "D3": (868, 1),
    "E1": (869, 4),
    "E2": (873, 4),
    "E3": (877, 4),
    "E4": (881, 2),
    "F1": (883, 15),
    "F2": (898, 162),
    "F3": (1060, 162),
    "F4": (1222, 54),
    "F5": (1276, 54),
    "F6": (1330, 16),
    "G1": (1346, 19),
    "G2": (1365, 13),
    "G3": (1378, 4),
}

PHASE_BLOCK = STATE_BLOCKS["B1"]
PHASES = ("bidding", "contract choice", "king call", "talon pick", "discard", "announcements", "play", "over")
REVEALED_PARTNER_BLOCK = STATE_BLOCKS["B6"]
UNSEEN_BLOCK = STATE_BLOCKS["F4"]  # card is unseen: the card-location loss counts these cards only

# --- Section 5: actions ------------------------------------------------------

N_ACTIONS = 160

# name: (first slot, count)
ACTION_GROUPS = {
    "BID": (0, 13),
    "KING": (13, 4),
    "TALON": (17, 6),
    "DISCARD": (23, 54),
    "ANNOUNCE": (77, 5),
    "KONTRA": (82, 6),
    "ANNOUNCE_DONE": (88, 1),
    "ANNOUNCE_BARVNI_VALAT": (89, 1),
    "PLAY": (90, 54),
    "RESERVED": (144, 16),
}

# --- Section 6: critic input ---------------------------------------------------

# name: width; these follow the state features in the critic array
CRITIC_EXTRA_BLOCKS = {
    "other hands": 162,
    "talon not public": 54,
    "hidden discards": 54,
    "true partner seat": 4,
}
CRITIC_EXTRA = 274
CRITIC_FEATURES = STATE_FEATURES + CRITIC_EXTRA

# --- Section 7: auxiliary targets ----------------------------------------------

# The contract names the targets but not their number codes. The learner fixes
# them here; the bindings (spec 006) must emit the same codes.
#
# Partner seat: the relative seat of the partner, 1 to 3. 0 means "none"; the
# loss skips those decisions. The head has one logit per relative seat.
PARTNER_CLASSES = 4
PARTNER_NONE = 0
# Card location, one class per card: 0 = out of play (talon rest, discards),
# 1 to 3 = the other seat with that relative index.
CARD_LOCATION_CLASSES = 4

# --- Section 8: reward ---------------------------------------------------------

REWARD_SCALE = 50
# The score parts of section 8 (D121), in the contract's order. The decomposed value
# predicts them. The environment gives `reward_parts`, one number per seat and part,
# at the end of a deal; the parts of a seat add up to its reward. The last two
# (klop, radelc) are zero until flow C of the engine.
SCORE_PARTS = (
    "game",
    "difference",
    "trula",
    "kralji",
    "kralj ultimo",
    "pagat ultimo",
    "valat",
    "mondfang",
    "klop",
    "radelc",
)
N_SCORE_PARTS = len(SCORE_PARTS)
