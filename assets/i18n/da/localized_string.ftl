shared_photos =
    {$userName} {$photoCount ->
        [one] tilføjede et nyt billede
        *[other] tilføjede {$photoCount} nye billeder
    } til {$userGender ->
        [male] hans væg
        [female] hendes væg
        *[other] deres væg
    }.
username = Brugernavn:
gender = Køn:
    .male = Mand
    .female = Kvinde
    .other = Andet
photo_count = Antal billeder:
locale = Sprog: