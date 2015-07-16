-- test
x = 0
--print_ob(foo["ob"])
yop = foo["ob"]
metatable = getmetatable(yop)
ppp = yop.position
metatablepos = getmetatable(ppp)
--metatable.__index = metatable
--yop.__metatable = metatable
--print("metatable : ", metatable, " \n");
object.print_ob(yop)
--yop.get_pos(yop)
x,y,z = object.get_pos(yop)
--caca = yop:__to_string()
yop.position.x = yop.position.x + 0.1
yosh = vec3.new(7,8,9)
yosh2 = vec3.new(17,8,9)
yosh.x = 2
yop.position = yosh + yosh2
print("yosh : ", yosh.x)
print("caca : ", yop.position.x)
--yop["xxx"] = -140
--yop.y = 10
for n,v in pairs(metatablepos) do print(n,v) end
print("aida")
for n,v in pairs(metatable) do print(n,v) end
print("func start \n");
--for n,v in pairs(object) do print(n,v) end
print("func end \n");
io.write("Just doing nothing \n");
print("what is my pos :: ",  x,y,z, "\n");
return x
