# internal
mults = [
    ["1[1]", "2(∞)[1]", True],
    ["1[3]", "2(∞)[3]", True],
    ["1[5]", "2(∞)[5]", True],
    ["1[7]", "2(∞)[7]", True],





    ["h_2[2]", "h_0h_2[2]", True],
    ["h_2[3]", "h_0h_2[3]", True],
    ["h_2[4]", "h_0h_2[4]", True],

    ["h_0h_2[1]", "h_0^2h_2[1]", True],
    ["h_0h_2[2]", "h_0^2h_2[2]", True],
    ["h_0h_2[3]", "h_0^2h_2[3]", True],
    ["h_0h_2[4]", "h_0^2h_2[4]", True],

]

# External
mults += [
    ["1[3]", "h_1[2]", False],
    ["h_1[2]", "h_1^2[1]", False],

    ["1[7]", "h_1[6]", False],
    ["h_1[6]", "h_1^2[5]", False],
    ["h_1^2[5]", "h_0^2h_2[4]", False],


]




strings = []
for m in mults:
    a = "true" if m[2] else "false"
    a = "{\"from\": \"" + m[0] + "\", \"to\": \"" + m[1] + "\", \"internal\": " + a + "}"
    strings.append(a)

print(",\n".join(strings))



