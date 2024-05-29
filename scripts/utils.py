
# rgba in [0, 255] -> [0, 1]
rgba_to_float = lambda rgba: [x / 255 for x in rgba]
# rgba tuple (r, g, b, a) in [0, 1], hue in [0, 1]
# rgba_add_hue = lambda rgba, hue_offset: (*sns.color_palette([rgba], 1, 1)[0], rgba[3])
def adjust_hue(rgba, hue_offset):
    """
    Adjusts the hue of an RGBA color.
    rgba: A tuple of (r, g, b, a) where each component is between 0 and 1.
    hue_offset: Hue offset in degrees (0 to 360).
    Returns: A new RGBA color with adjusted hue.
    """
    # Convert hue offset from degrees to a scale of 0-1
    hue_offset = hue_offset / 360.0

    # Unpack the RGBA values
    r, g, b, a = rgba

    # Convert RGB to HSV
    h, s, v = colorsys.rgb_to_hsv(r, g, b)

    # Adjust hue, ensuring it wraps around
    h = (h + hue_offset) % 1.0

    # Convert back to RGB
    r, g, b = colorsys.hsv_to_rgb(h, s, v)

    # Return the new color with the original alpha value
    return (r, g, b, a)

def set_alpha(rgba, alpha):
    return (*rgba[:3], alpha)

def lighten(rgba, factor):
    return (*[x + (1 - x) * factor for x in rgba[:3]], rgba[3])
    # return ([x + (1 - x) * factor for x in rgba[:3]])

def hex_to_rgba(hex):
    return rgba_to_float([int(hex[i:i+2], 16) for i in (1, 3, 5)])

# lighten hex
def lighten_hex(hex, factor):
    rgba = hex_to_rgba(hex)
    return rgba_to_hex(lighten(rgba, factor))

def darken(rgba, factor):
    return (*[x - x * factor for x in rgba[:3]], rgba[3])

def rgba_to_hex(rgba):
    return '#' + ''.join(f'{int(x * 255):02x}' for x in rgba[:3])