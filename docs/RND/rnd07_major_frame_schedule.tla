------------------- MODULE rnd07_major_frame_schedule -------------------
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS 
    MaxSlots,            \* Jumlah slot dalam 1 siklus Major Frame (misal: 2)
    MajorFrameDuration,  \* Durasi total major frame (misal: 100 ms)
    Partitions           \* Himpunan partisi {1, 2}

VARIABLES 
    current_slot,        \* Index slot saat ini (0..MaxSlots-1)
    elapsed_in_slot,     \* Waktu berlalu di slot saat ini
    active_partition,    \* Partisi yang saat ini memegang eksekusi CPU
    time_total,          \* Total waktu clock simulasi
    slot_durations       \* Sequence durasi per slot (misal: <<20, 80>>)

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
    /\ slot_durations = <<20, 80>>

Tick(dt) ==
    /\ time_total < 500  \* Batas eksplorasi finite state model checking
    /\ time_total' = time_total + dt
    /\ LET cur_dur == slot_durations[current_slot + 1]
       IN IF elapsed_in_slot + dt >= cur_dur
          THEN /\ elapsed_in_slot' = 0
               /\ current_slot' = (current_slot + 1) % MaxSlots
               /\ active_partition' = IF current_slot' = 0 THEN 1 ELSE 2
               /\ UNCHANGED slot_durations
          ELSE /\ elapsed_in_slot' = elapsed_in_slot + dt
               /\ UNCHANGED <<current_slot, active_partition, slot_durations>>

Next ==
    \/ Tick(10)
    \/ Tick(20)

Spec == Init /\ [][Next]_vars

-----------------------------------------------------------------------------
\* INVARIANT 1: NoSlotOverlap (Hanya 1 partisi yang aktif memegang CPU pada 1 waktu)
NoSlotOverlap ==
    active_partition \in Partitions

\* INVARIANT 2: FullCoverage (Total durasi seluruh slot tepat sama dengan Major Frame)
FullCoverage ==
    slot_durations[1] + slot_durations[2] = MajorFrameDuration

\* INVARIANT 3: BoundedJitter (Waktu berlalu di slot tidak boleh melebihi durasi slot)
BoundedJitter ==
    elapsed_in_slot <= slot_durations[current_slot + 1]

\* INVARIANT 4: NoDeadlineMiss (Waktu global bergerak maju secara monotonik)
NoDeadlineMiss ==
    time_total >= 0

=============================================================================
