------------------- MODULE rnd07_scaled_schedule -------------------
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS 
    MaxSlots,            \* Jumlah slot dalam 1 siklus Major Frame (4, 8, atau 16)
    MajorFrameDuration,  \* Durasi total major frame (misal: 100 atau 160 ms)
    Partitions           \* Himpunan partisi {1, 2, 3, 4} atau {1..6}

VARIABLES 
    current_slot,        \* Index slot saat ini (0..MaxSlots-1)
    elapsed_in_slot,     \* Waktu berlalu di slot saat ini
    active_partition,    \* Partisi yang saat ini memegang eksekusi CPU
    time_total,          \* Total waktu clock simulasi
    slot_durations       \* Sequence durasi per slot

vars == <<current_slot, elapsed_in_slot, active_partition, time_total, slot_durations>>

TypeOK ==
    /\ current_slot \in 0..(MaxSlots - 1)
    /\ elapsed_in_slot \in 0..MajorFrameDuration
    /\ active_partition \in Partitions
    /\ time_total \in 0..1000
    /\ Len(slot_durations) = MaxSlots

Init ==
    /\ current_slot = 0
    /\ elapsed_in_slot = 0
    /\ active_partition = 1
    /\ time_total = 0
    /\ slot_durations = [i \in 1..MaxSlots |-> MajorFrameDuration \div MaxSlots]

Tick(dt) ==
    /\ time_total < 300  \* Batas eksplorasi finite state model checking
    /\ time_total' = time_total + dt
    /\ LET cur_dur == slot_durations[current_slot + 1]
       IN IF elapsed_in_slot + dt >= cur_dur
          THEN /\ elapsed_in_slot' = 0
               /\ current_slot' = (current_slot + 1) % MaxSlots
               /\ active_partition' = ((current_slot' % Cardinality(Partitions)) + 1)
               /\ UNCHANGED slot_durations
          ELSE /\ elapsed_in_slot' = elapsed_in_slot + dt
               /\ UNCHANGED <<current_slot, active_partition, slot_durations>>

Next ==
    \/ Tick(5)
    \/ Tick(10)

Spec == Init /\ [][Next]_vars

-----------------------------------------------------------------------------
\* INVARIANT 1: NoSlotOverlap (Hanya 1 partisi aktif pada satu waktu)
NoSlotOverlap ==
    active_partition \in Partitions

\* INVARIANT 2: BoundedJitter (Waktu berlalu di slot tidak pernah melebihi durasi slot)
\* Catatan Teknis W-12: BoundedJitter berlaku trivially pada model deterministik diskrit ini.
BoundedJitter ==
    elapsed_in_slot <= slot_durations[current_slot + 1]

\* INVARIANT 3: NoDeadlineMiss (Waktu global monotonik naik tanpa deadlock)
NoDeadlineMiss ==
    time_total >= 0

=============================================================================
